use std::{collections::HashMap, future::Ready, io::Read, ops::Sub, path::PathBuf};
use thiserror::Error;
use zenoh::{Resolvable, bytes::ZBytes, handlers::{Callback, DefaultHandler, FifoChannelHandler}, query::{Query, Queryable, QueryableBuilder}, sample::{Sample, SampleKind}};

#[derive(Error, Debug)]
pub enum EvoBridgeError {
    #[error("All senders have been dropped")]
    ALL_SENDERS_DROPPED,
    #[error("Prost encoding error")]
    ENCODING_ERROR,
    #[error("Prost decoding error")]
    DECODING_ERROR,
    #[error("There was no content in the payload")]
    NO_CONTENT
}

enum SubCallback<I: Fn() + Send + Sync + 'static, F: FnMut() + Send + Sync + 'static> {
    NONE,
    IMMUTABLE(I),
    MUTABLE(F)
}

pub struct Session {
    session: zenoh::Session, 
    remappings: HashMap<String, String>,
    namespace: Option<String>
}

impl Session {
    ///Function to create new session, will panic is opening the zenoh session fails
    pub async fn new(remappings: Option<HashMap<String, String>>, namespace: Option<String> ) -> Self {

        //TODO: may want to consider adding a way to update the zenoh config. Not important for now
        //though
        Self {
            session: zenoh::open(zenoh::Config::default()).await.unwrap(),
            remappings: HashMap::new(),
            namespace: None
        }
    }

    ///remaps the inputed topic adding a namespace if it exists and applying remappings if needed.
    fn remap(&self, value: String) -> String {
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
    
    pub fn subscribe(&self, key: &str) -> SubscriberBuilder<DefaultHandler> {
        let builder = self.session.declare_subscriber(self.remap(key.to_string()));
        SubscriberBuilder {
            builder: builder         
        }
    }

    pub async fn get_publisher<'a>(&self, key: &str) -> Publisher {
        //idk why but docs say to unwrap and fail to say what case this fails in so here we are
        let publisher = self.session.declare_publisher(self.remap(key.to_string())).await.unwrap();
        Publisher {
            publisher
        }
    }

    pub async fn declare_service(&self, key: &str) {
        let service = self.session.declare_queryable(self.remap(key.to_string()));
        unimplemented!()
    }

    pub async fn declare_service_caller(&self, key: &str) {
        unimplemented!()
    }

    //TODO: implement services
}

fn decode<T: prost::Message + Default>(sample: ZBytes) -> Result<T, EvoBridgeError> {
    let mut buf = Vec::new();
    let mut reader = sample.reader();
    let _ = reader.read(&mut buf);
    match T::decode(&*buf) {
        Ok(d) => Ok(d),
        Err(e) => Err(EvoBridgeError::DECODING_ERROR)
    }
}

pub struct SubMessage<T: prost::Message> {
    payload: T
}

pub struct SubscriberBuilder<'a, H> {
    builder: zenoh::pubsub::SubscriberBuilder<'a,'a, H>
}

impl <'a> SubscriberBuilder<'a, DefaultHandler> {
    pub fn with_mut_callback<T: prost::Message + Default, F: FnMut(Result<SubMessage<T> ,EvoBridgeError>) + Send + Sync + 'static>(self, mut callback: F) -> SubscriberBuilder<'a, Callback<Sample>>{
        let builder = self.builder.callback_mut(move |m| {
            //TODO: Clone is probably a performance issue, pls fix
            let result = match decode(m.payload().clone()) {
                Ok(p) => Ok(SubMessage {
                    payload: p
                }),
                Err(e) => Err(EvoBridgeError::DECODING_ERROR)
            };
            callback(result);
        });
        SubscriberBuilder::<'a, Callback<Sample>> { builder: builder }
    }

    pub fn with_callback<T: prost::Message + Default, F: Fn(Result<SubMessage<T> ,EvoBridgeError>) + Send + Sync + 'static>(self, callback: F) -> SubscriberBuilder<'a, Callback<Sample>> {
        //TODO: Clone is probably a performance issue, pls fix
        let builder = self.builder.callback(move |m| {
            let result = match decode(m.payload().clone()) {
                Ok(p) => Ok(SubMessage {
                    payload: p
                }),
                Err(e) => Err(EvoBridgeError::DECODING_ERROR)
            };
            callback(result);
        });
        SubscriberBuilder::<'a, Callback<Sample>> { builder: builder }
    }
}
    
pub struct Subscriber<H> {
    subscriber: zenoh::pubsub::Subscriber<H>
}

impl Subscriber<FifoChannelHandler<Sample>> {
    pub async fn recv_async<T: prost::Message + std::default::Default>(&self) -> Result<T, EvoBridgeError> {
        let sample = match self.subscriber.recv_async().await {
            Ok(p) => p,
            Err(e) => return Err(EvoBridgeError::ALL_SENDERS_DROPPED)
        };
        
        //TODO: Replace with SubMessage
        let mut buf = Vec::new();
        let mut reader = sample.payload().reader();
        reader.read(&mut buf);
        match T::decode(&*buf) {
            Ok(d) => Ok(d),
            Err(e) => Err(EvoBridgeError::DECODING_ERROR)
        }

    }
}

pub struct Publisher<'a> {
    publisher: zenoh::pubsub::Publisher<'a>
}

impl<'a> Publisher<'a> {
    pub async fn put<T: prost::Message>(&self, message: T) -> Result<(), EvoBridgeError>{
        let mut buf = Vec::new();
        let _ = message.encode(&mut buf).unwrap_or(return Err(EvoBridgeError::ENCODING_ERROR));
        self.publisher.put(buf).await.unwrap();
        Ok(())
    } 
    
    //will do this later
    // pub async fn get_matching_listener(&self) {
    //     self.publisher.matching_listener().cal.await.unwrap();
    // }

    //TODO: matching listener function.
}


pub struct ServiceCall {
    call: Query
}

impl ServiceCall {
    pub fn get_payload<T: prost::Message + Default>(&self) -> Result<T, EvoBridgeError> {
        //TODO: Clone here
        let payload = match self.call.payload() {
            Some(k) => k.clone(),
            None => return Err(EvoBridgeError::NO_CONTENT)
        };

        match decode::<T>(payload) {
            Ok(p) => Ok(p),
            Err(e) => Err(EvoBridgeError::DECODING_ERROR)
        }
    }

    pub fn get_attachment<T: prost::Message + Default>(&self) -> Result<T, EvoBridgeError> {
        //TODO: clone here
        let payload = match self.call.attachment() {
            Some(k) => k.clone(),
            None => return Err(EvoBridgeError::NO_CONTENT)
        };

        match decode::<T>(payload) {
            Ok(p) => Ok(p),
            Err(e) => Err(EvoBridgeError::DECODING_ERROR)
        }
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

