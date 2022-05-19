#[derive(Clone, PartialEq, ::prost::Message)]
pub struct Void {
}
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct Sensor {
    #[prost(int32, tag="1")]
    pub id: i32,
}
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct SensorValue {
    #[prost(float, tag="1")]
    pub value: f32,
}
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct Zigbee2MqttConfig {
    #[prost(string, tag="1")]
    pub url: ::prost::alloc::string::String,
    #[prost(string, tag="2")]
    pub username: ::prost::alloc::string::String,
    #[prost(string, tag="3")]
    pub password: ::prost::alloc::string::String,
}
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct Zigbee2MqttStats {
    #[prost(int32, tag="1")]
    pub sources: i32,
    #[prost(int32, tag="2")]
    pub sinks: i32,
    #[prost(int32, tag="3")]
    pub sourcesinks: i32,
    #[prost(int32, tag="4")]
    pub devices: i32,
}
#[async_trait::async_trait]
pub trait Zigbee2MqttService {
    
        
            async fn config(input: Zigbee2MqttConfig) -> anyhow::Result<Zigbee2MqttStats>;
        
    
        
            async fn status(input: Void) -> anyhow::Result<Zigbee2MqttStats>;
        
    
}

#[derive(Default)]
pub struct Zigbee2MqttServiceRouter<T> {
    _marker: std::marker::PhantomData<T>,
}

#[async_trait::async_trait]
impl<T> super::ServiceRouter for Zigbee2MqttServiceRouter<T> where T: Zigbee2MqttService + Send + Sync + Default {
    fn id(&self) -> u16 {
        0xd109
    }

    async fn route(&self, req: crate::http::pipe::PipeRequest, tx: tokio::sync::mpsc::Sender<Vec<u8>>) -> anyhow::Result<()> {
        use prost::Message as _;

        match req.call_id {
            
                0xd74e => {
                    
                        let input = Zigbee2MqttConfig::decode(req.payload_slice())?;
                        let data = T::config(input).await?;
                        let out = crate::http::pipe::response_message(req.channel_id, data)?;
                        tx.send(out).await?;

                        Ok(())
                    
                },
            
                0x9b83 => {
                    
                        let input = Void::decode(req.payload_slice())?;
                        let data = T::status(input).await?;
                        let out = crate::http::pipe::response_message(req.channel_id, data)?;
                        tx.send(out).await?;

                        Ok(())
                    
                },
            
            _ =>  anyhow::bail!("invalid call id {}", req.call_id),
        }
    }
}
#[async_trait::async_trait]
pub trait StatisticsService {
    
        
            async fn listen(input: Sensor) -> anyhow::Result<tokio::sync::mpsc::Receiver<anyhow::Result<SensorValue>>>;
        
    
}

#[derive(Default)]
pub struct StatisticsServiceRouter<T> {
    _marker: std::marker::PhantomData<T>,
}

#[async_trait::async_trait]
impl<T> super::ServiceRouter for StatisticsServiceRouter<T> where T: StatisticsService + Send + Sync + Default {
    fn id(&self) -> u16 {
        0x5f00
    }

    async fn route(&self, req: crate::http::pipe::PipeRequest, tx: tokio::sync::mpsc::Sender<Vec<u8>>) -> anyhow::Result<()> {
        use prost::Message as _;

        match req.call_id {
            
                0x47a5 => {
                    
                        let input = Sensor::decode(req.payload_slice())?;
                        let mut rx = T::listen(input).await?;
                        while let Some(data) = rx.recv().await {
                            let out = match data {
                                Ok(data) => crate::http::pipe::part_message(req.channel_id, data)?,
                                Err(e) => crate::http::pipe::error_message(req.channel_id, &format!("{}", e)),
                            };
                            tx.send(out).await?;
                        }
                        Ok(())
                    
                },
            
            _ =>  anyhow::bail!("invalid call id {}", req.call_id),
        }
    }
}
