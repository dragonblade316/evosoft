use std::{collections::HashMap, fmt::Display, io::Read };
use derive_name::{Name, Named};
use thiserror::Error;
use zenoh::{Resolvable, bytes::ZBytes, handlers::{Callback, DefaultHandler, FifoChannelHandler}, matching::MatchingStatus, query::{Querier, Query, Queryable, QueryableBuilder, Reply}, sample::{Sample, SampleKind}};

#[derive(Error, Debug)]
pub enum EvoBridgeError {
    #[error("All senders have been dropped")]
    ALL_SENDERS_DROPPED,
    #[error("Prost encoding error")]
    ENCODING_ERROR,
    #[error("Prost decoding error")]
    DECODING_ERROR,
    #[error("There was no content in the payload")]
    NO_CONTENT,
    #[error("Reply error")]
    REPLY_ERROR(zenoh::query::ReplyError)
}

pub struct Session {
    session: zenoh::Session, 
    remapper: Remapper
}

impl Session {
    ///Function to create new session, will panic is opening the zenoh session fails
    pub async fn new(remappings: Option<HashMap<String, String>>, namespace: Option<String> ) -> Self {
        //TODO: may want to consider adding a way to update the zenoh config. Not important for now
        //though
        Self {
            session: zenoh::open(zenoh::Config::default()).await.unwrap(),
            remapper: Remapper { remappings: remappings.unwrap_or(HashMap::new()), namespace: namespace }
        }
    }

    pub fn subscribe(&self, key: &str) -> SubscriberBuilder<DefaultHandler> {
        let builder = self.session.declare_subscriber(self.remapper.remap_with_namespace(key.to_string()));
        SubscriberBuilder {
            builder: builder         
        }
    }

    pub async fn get_publisher<'a>(&self, key: &str) -> Publisher {
        //idk why but docs say to unwrap and fail to say what case this fails in so here we are
        let publisher = self.session.declare_publisher(self.remapper.remap(key.to_string())).await.unwrap();
        Publisher {
            publisher
        }
    }

    pub async fn declare_service(&self, key: &str) -> ServiceBuilder<'_, DefaultHandler> {
        let service = self.session.declare_queryable(self.remapper.remap_with_namespace(key.to_string()));
        ServiceBuilder {
            builder: service 
        }
    }

    pub async fn declare_service_caller(&self, key: &str) -> ServiceCaller<'_> {
        let caller = self.session.declare_querier(self.remapper.remap(key.to_string())).await.unwrap();
        ServiceCaller {
            service: caller,
            remapper: self.remapper.clone()
        }
    }
}

#[derive(Debug, Clone)]
struct Remapper {
    remappings: HashMap<String, String>,
    namespace: Option<String> //TODO: Verify this is not needed and remove
}

impl Remapper {
    fn remap(&self, value: String) -> String {
        let topic_name = match self.remappings.contains_key(&value) {
            // true -> self.remappings.get(&value).expect("We gurenteed that the key exists").clone();
            true => self.remappings.get(&value).expect("We gurenteed that the key exists").clone(),
            false => value,
        };

        // match &self.namespace {
        //     Some(namespace) => format!("{}/{}", namespace, topic_name),
        //     None => topic_name
        // } 
        topic_name
    }


    pub fn remap_with_namespace(&self, value: String) -> String {
        let topic_name = match self.remappings.contains_key(&value) {
            // true -> self.remappings.get(&value).expect("We gurenteed that the key exists").clone();
            true => self.remappings.get(&value).expect("We gurenteed that the key exists").clone(),
            false => value,
        };

        match &self.namespace {
            Some(namespace) => format!("{}/{}", namespace, topic_name),
            None => topic_name
        }
    }
}

fn decode<T: prost::Message + Default>(sample: &ZBytes) -> Result<T, EvoBridgeError> {
    let mut buf = Vec::new();
    let mut reader = sample.reader();
    reader.read_to_end(&mut buf).expect("Reading zbytes failed. I have no idea how this can happen");
    match T::decode(&*buf) {
        Ok(d) => Ok(d),
        Err(e) => Err(EvoBridgeError::DECODING_ERROR)
    }
}

fn encode<T: prost::Message + Default>(data: &T) -> Vec<u8> {
    data.encode_to_vec()
}

#[derive(Debug)]
pub struct SubMessage {
    payload: ZBytes,
    encoding: zenoh::bytes::Encoding
}

impl Display for SubMessage {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "Encoding is: {}. \n ZByte array is {} elements long.", self.encoding().to_string(), self.payload.len()) 
    } 
}

impl SubMessage {
    pub fn payload<T: prost::Message + Default>(&self) -> Result<T, EvoBridgeError> {
        decode(&self.payload)
    } 

    pub fn encoding(&self) -> zenoh::bytes::Encoding {
        self.encoding.clone() 
    }
}

pub struct SubscriberBuilder<'a, H> {
    builder: zenoh::pubsub::SubscriberBuilder<'a,'a, H>
}

impl <'a> SubscriberBuilder<'a, DefaultHandler> {
    pub fn with_mut_callback<F: FnMut(SubMessage) + Send + Sync + 'static>(self, mut callback: F) -> SubscriberBuilder<'a, Callback<Sample>>{
        let builder = self.builder.callback_mut(move |m| {
            //TODO: Clone is probably a performance issue, pls fix
            // let result = match decode(m.payload().clone()) {
            //     Ok(p) => Ok(SubMessage {
            //         payload: p,
            //         encoding: m.encoding().clone()
            //     }),
            //     Err(e) => Err(EvoBridgeError::DECODING_ERROR)
            // }; TODO: prob gonna have to delete this
            let result = SubMessage {
                payload: m.payload().clone(),
                encoding: m.encoding().clone()
            };
            callback(result);
        });
        SubscriberBuilder::<'a, Callback<Sample>> { builder: builder }
    }

    pub fn with_callback<F: Fn(SubMessage) + Send + Sync + 'static>(self, callback: F) -> SubscriberBuilder<'a, Callback<Sample>> {
        //TODO: Clone is probably a performance issue, pls fix
        let builder = self.builder.callback(move |m| {
            // let result = match decode(m.payload().clone()) {
            //     Ok(p) => Ok(SubMessage {
            //         payload: p,
            //         encoding: m.encoding().clone()
            //     }),
            //     Err(e) => Err(EvoBridgeError::DECODING_ERROR)
            // }; //TODO: Test then delete this comment
            let result = SubMessage {
                payload: m.payload().clone(),
                encoding: m.encoding().clone()
            };
            callback(result);
        });
        SubscriberBuilder::<'a, Callback<Sample>> { builder: builder }
    }

    pub async fn build(self) -> Subscriber<FifoChannelHandler<Sample>> {
        let sub = self.builder.await.expect("Error cases are not currently known so they are being ignored, if a case is discovered, remove this expect");
        
        Subscriber {
            subscriber: sub
        }
    }
}

//there is probably an easier way to do this than doing two methods but I am tired of fighting the
//type system and there are only two possible cases so this is fine.
impl<'a> SubscriberBuilder<'a, Callback<Sample>> {
    pub async fn build(self) -> Subscriber<()> {
        let sub = self.builder.await.expect("Error cases are not currently known so they are being ignored, if a case is discovered, remove this expect");
        
        Subscriber {
            subscriber: sub
        }
    }
}
    
pub struct Subscriber<H> {
    subscriber: zenoh::pubsub::Subscriber<H>
}

impl Subscriber<FifoChannelHandler<Sample>> {
    pub async fn recv_async(&self) -> Result<SubMessage, EvoBridgeError> {
        let sample = match self.subscriber.recv_async().await {
            Ok(p) => p,
            Err(e) => return Err(EvoBridgeError::ALL_SENDERS_DROPPED)
        };

        //TODO: remove these clones
        Ok(SubMessage {
            payload: sample.payload().clone(),
            encoding: sample.encoding().clone()
        })

        
        //TODO: Replace with SubMessage
        // let mut buf = Vec::new();
        // let mut reader = sample.payload().reader();
        // let _ = reader.read(&mut buf);
        // match T::decode(&*buf) {
        //     Ok(d) => Ok(d),
        //     Err(e) => Err(EvoBridgeError::DECODING_ERROR)
        // } TODO: Delete

    }
}

pub struct MatchingListenerBuilder<'a, H> {
    builder: zenoh::matching::MatchingListenerBuilder<'a, H>
}

impl <'a> MatchingListenerBuilder<'a, DefaultHandler> {
    pub fn with_mut_callback<F: FnMut(MatchingStatus) + Send + Sync + 'static>(self, mut callback: F) -> MatchingListenerBuilder<'a, Callback<MatchingStatus>>{
        let builder = self.builder.callback_mut(move |m| {
            callback(m);
        });
        MatchingListenerBuilder::<'a, Callback<MatchingStatus>> { builder: builder }
    }

    pub fn with_callback<F: Fn(MatchingStatus) + Send + Sync + 'static>(self, callback: F) -> MatchingListenerBuilder<'a, Callback<MatchingStatus>> {
        let builder = self.builder.callback(move |m| {
            callback(m);
        });
        MatchingListenerBuilder::<'a, Callback<MatchingStatus>> { builder: builder }
    }

    pub async fn build(self) -> MatchingListener<FifoChannelHandler<MatchingStatus>> {
        let listener = self.builder.await.expect("Error cases are not currently known so they are being ignored, if a case is discovered, remove this expect");
        
        MatchingListener {
            listener: listener
        }
    }
}

struct MatchingListener<H> {
    listener: zenoh::matching::MatchingListener<H>
} 

impl MatchingListener<FifoChannelHandler<MatchingStatus>> {
    pub async fn recv_async(&self) -> Result<MatchingStatus, EvoBridgeError> {
        match self.listener.recv_async().await {
            Ok(p) => Ok(p),
            Err(e) => return Err(EvoBridgeError::ALL_SENDERS_DROPPED)
        }
    }
}

//there is probably an easier way to do this than doing two methods but I am tired of fighting the
//type system and there are only two possible cases so this is fine.
//TODO: Figured it out. This is for the Matching Builder
// impl<'a> MatchingBuilder<'a, Callback<Sample>> {
//     pub async fn build(self) -> Result<MatchingStatus, EvoBridgeError>{
//         let sub = self.builder.await.expect("Error cases are not currently known so they are being ignored, if a case is discovered, remove this expect");
//         Subscriber {
//             subscriber: sub
//         }
//     }
// }


pub struct Publisher<'a> {
    publisher: zenoh::pubsub::Publisher<'a>
}

impl<'a> Publisher<'a> {
    //TODO: remove the result or figure out an error case
    pub async fn put<T: prost::Message + Default + Name>(&self, message: T) -> Result<(), EvoBridgeError>{
        self.publisher.put(encode(&message)).encoding(format!("protobuf/{}", T::name())).await.unwrap();
        Ok(())
    } 
    
    pub async fn get_matching_listener(&self) -> MatchingListenerBuilder<DefaultHandler> {
        let builder = self.publisher.matching_listener();
        MatchingListenerBuilder {
            builder: builder
        }
    }
}


pub struct ServiceCall {
    call: Query
}

impl ServiceCall {
    pub fn payload<T: prost::Message + Default>(&self) -> Result<T, EvoBridgeError> {
        //TODO: Clone here
        let payload = match self.call.payload() {
            Some(k) => k.clone(),
            None => return Err(EvoBridgeError::NO_CONTENT)
        };

        match decode::<T>(&payload) {
            Ok(p) => Ok(p),
            Err(e) => Err(EvoBridgeError::DECODING_ERROR)
        }
    }

    pub fn attachment<T: prost::Message + Default>(&self) -> Result<T, EvoBridgeError> {
        //TODO: clone here
        let payload = match self.call.attachment() {
            Some(k) => k.clone(),
            None => return Err(EvoBridgeError::NO_CONTENT)
        };
        
        match decode::<T>(&payload) {
            Ok(p) => Ok(p),
            Err(e) => Err(EvoBridgeError::DECODING_ERROR)
        }
    }

    pub fn encoding(&self) -> Option<zenoh::bytes::Encoding> {
        //TODO: this clone is probably fine but lets double check later
        self.call.encoding().cloned()
    }

    pub async fn reply<T: prost::Message>(&self, message: T) -> Result<(), EvoBridgeError>{
       
        
        let mut buf = Vec::new();
        let _ = message.encode(&mut buf).unwrap_or(return Err(EvoBridgeError::ENCODING_ERROR));

        // self.call.reply(key_expr, payload)
        
        // self.call.reply(remap(), payload);

        // self.publisher.put(buf).await.unwrap();
        unimplemented!()
    } 
}

pub struct ServiceBuilder<'a, H> {
    builder: QueryableBuilder<'a, 'a, H>
}

impl <'a> ServiceBuilder<'a, DefaultHandler> {
    pub fn with_mut_callback<F: FnMut(ServiceCall) + Send + Sync + 'static>(self, mut callback: F) -> ServiceBuilder<'a, Callback<Query>>{
        let builder = self.builder.callback_mut(move |m| {

            callback(ServiceCall { call: m });
        });
        ServiceBuilder::<'a, Callback<Query>> { builder: builder }
        
    }

    pub fn with_callback<F: Fn(ServiceCall) + Send + Sync + 'static>(self, callback: F) -> ServiceBuilder<'a, Callback<Query>> {
        //TODO: Clone is probably a performance issue, pls fix
        let builder = self.builder.callback(move |m| {
            callback(ServiceCall { call: m });
        });
        ServiceBuilder::<'a, Callback<Query>> { builder: builder }
    }

    //might want to add completeness and allowed queryers at some point

}

pub struct Service<H> {
    service: Queryable<H>
}

impl Service<FifoChannelHandler<Query>> {
    pub async fn recv_async(&self) -> Result<ServiceCall, EvoBridgeError> {
        let query = match self.service.recv_async().await {
            Ok(p) => p,
            //I dont think this error state is possible but I'm too lazy to remove it
            Err(e) => return Err(EvoBridgeError::ALL_SENDERS_DROPPED)
        };

        Ok(ServiceCall {
            call: query 
        })
    }
}




pub struct QueryBuilder<'a, H> {
    builder: zenoh::query::QuerierGetBuilder<'a, 'a, H>,
    remapper: Remapper
}

impl <'a> QueryBuilder<'a, DefaultHandler> {
    pub fn with_mut_callback<T: prost::Message + Default, F: FnMut(SubMessage) + Send + Sync + 'static>(self, mut callback: F) -> QueryBuilder<'a, Callback<Reply>>{
        let builder = self.builder.callback_mut(move |m| {
            //TODO: Clone is probably a performance issue, pls fix
            let data = m.result().unwrap();
            let result = SubMessage { payload: data.payload().clone(), encoding: data.encoding().clone()  };
            callback(result);
        });
        QueryBuilder::<'a, Callback<Reply>> { builder: builder, remapper: self.remapper}
    }

    pub fn with_callback<T: prost::Message + Default, F: Fn(SubMessage) + Send + Sync + 'static>(self, callback: F) -> QueryBuilder<'a, Callback<Reply>> {
        //TODO: Clone is probably a performance issue, pls fix
        let builder = self.builder.callback(move |m| {
            let data = m.result().unwrap();
            let result = SubMessage { payload: data.payload().clone(), encoding: data.encoding().clone()  };
           
            callback(result);
        });
        QueryBuilder::<'a, Callback<Reply>> { builder: builder, remapper: self.remapper}
    }

    pub async fn build(self) -> OutgoingServiceCall<FifoChannelHandler<Reply>> {
        let query = self.builder.await.expect("Error cases are not currently known so they are being ignored, if a case is discovered, remove this expect");

        OutgoingServiceCall {
            query
        }
    }
}

impl<'a, H> QueryBuilder<'a, H> {
    pub fn payload<T: prost::Message + Default + Name>(self, payload: T) -> QueryBuilder<'a, H> {
        let builder = self.builder.payload(encode(&payload)).encoding(format!("protobuf/{}", payload.name()));
        QueryBuilder::<'a, H> {builder: builder, remapper: self.remapper}
    }

    pub fn attachment<T: prost::Message + Default>(self, payload: T) -> QueryBuilder<'a, H> {
        let builder = self.builder.attachment(encode(&payload));
        QueryBuilder::<'a, H> {builder: builder, remapper: self.remapper}
    }
}

//there is probably an easier way to do this than doing two methods but I am tired of fighting the
//type system and there are only two possible cases so this is fine.
impl<'a> QueryBuilder<'a, Callback<Reply>> {
    pub async fn build(self) -> OutgoingServiceCall<()> {
        let query = self.builder.await.expect("Error cases are not currently known so they are being ignored, if a case is discovered, remove this expect");

        OutgoingServiceCall {
            query 
        }
    }
}

pub struct OutgoingServiceCall<H> {
    query: H
}

impl OutgoingServiceCall<FifoChannelHandler<Reply>> {
     pub async fn recv_async(&self) -> Result<serviceReply, EvoBridgeError> {
        let query = match self.query.recv_async().await {
            Ok(p) => p,
            //I dont think this error state is possible but I'm too lazy to remove it
            Err(e) => return Err(EvoBridgeError::ALL_SENDERS_DROPPED)
        };

        //add reply wrapper
        Ok(serviceReply {
            reply: query 
        })
    }
}

//TODO: Rename this to reply
pub struct serviceReply {
    reply: zenoh::query::Reply
}

impl serviceReply {
    pub fn payload<T: prost::Message + Default>(&self) -> Result<T, EvoBridgeError> {
        let data = match self.reply.result() {
            Ok(k) => k,
            Err(e) => return Err(EvoBridgeError::REPLY_ERROR(e.clone()))
        };
        
        //TODO: clone here to maybe be removed
        Ok(decode(&data.payload().clone()).unwrap_or(return Err(EvoBridgeError::DECODING_ERROR)))
    }

    pub fn encoding(&self) -> Result<zenoh::bytes::Encoding, EvoBridgeError> {
        let data = match self.reply.result() {
            Ok(k) => k,
            Err(e) => return Err(EvoBridgeError::REPLY_ERROR(e.clone()))
        };

        Ok(data.encoding().clone())
    }
}


pub struct ServiceCaller<'a> {
    service: Querier<'a>,
    remapper: Remapper
}

//idk what '_ is doing here and tbh I am to scared to ask. but the borrow checker is happy so
//mission acomplised ig
impl<'a> ServiceCaller<'a> {
    pub async fn get(&self) -> QueryBuilder<'_, DefaultHandler> {
        let builder = self.service.get();
        QueryBuilder {
            builder,
            remapper: self.remapper.clone()
        }
    } 

    pub fn get_matching_listener(&self) -> MatchingListenerBuilder<DefaultHandler> {
        let builder = self.service.matching_listener();
        MatchingListenerBuilder {
            builder: builder
        }
    }
}

