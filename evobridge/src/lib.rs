use std::{collections::HashMap, io::Read, ops::Sub, path::PathBuf};
use thiserror::Error;
use zenoh::{handlers::FifoChannelHandler, sample::Sample};

#[derive(Error, Debug)]
pub enum EvoBridgeError {
    #[error("All senders have been dropped")]
    ALL_SENDERS_DROPPED,
    #[error("Prost encoding error")]
    ENCODING_ERROR,
    #[error("Prost decoding error")]
    DECODING_ERROR,
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
    
    pub async fn subscribe(&self) -> Subscriber {
        let subscriber = self.session.declare_subscriber(self.remap("".to_string())).await.unwrap();
        Subscriber {
            subscriber
        }
    }

    pub async fn get_publisher<'a>(&self) -> Publisher {
        //idk why but docs say to unwrap and fail to say what case this fails in so here we are
        let publisher = self.session.declare_publisher(self.remap("".to_string())).await.unwrap();
        Publisher {
            publisher
        }
    }

    //TODO: implement services
}

pub struct Subscriber {
    subscriber: zenoh::pubsub::Subscriber<FifoChannelHandler<Sample>>
}

impl Subscriber {
    pub async fn recv_async<T: prost::Message + std::default::Default>(&self) -> Result<T, EvoBridgeError> {
        let sample = match self.subscriber.recv_async().await {
            Ok(p) => p,
            Err(e) => return Err(EvoBridgeError::ALL_SENDERS_DROPPED)
        };
        
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

