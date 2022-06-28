#[derive(Clone, PartialEq, ::prost::Message)]
pub struct Void {
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
pub struct Zigbee2MqttServer {
    #[prost(string, tag="1")]
    pub host: ::prost::alloc::string::String,
    #[prost(uint32, tag="2")]
    pub port: u32,
    #[prost(uint32, tag="3")]
    pub devices: u32,
}
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct Zigbee2MqttServers {
    #[prost(message, repeated, tag="1")]
    pub servers: ::prost::alloc::vec::Vec<Zigbee2MqttServer>,
}
#[async_trait::async_trait]
pub trait Zigbee2MqttService {
    
        
            async fn config(ctx: crate::actor::Task, input: Zigbee2MqttConfig) -> anyhow::Result<Zigbee2MqttServer>;
        
    
        
            async fn status(ctx: crate::actor::Task, input: Void) -> anyhow::Result<Zigbee2MqttServers>;
        
    
}

#[derive(Default)]
pub struct Zigbee2MqttServiceRouter<T> {
    _marker: std::marker::PhantomData<T>,
}

impl<T> super::ServiceRouter for Zigbee2MqttServiceRouter<T> where T: Zigbee2MqttService + Send + Sync + Default {
    fn id(&self) -> u16 {
        0xd109
    }

    fn route(&self, ctx: crate::actor::Task, req: crate::http::pipe::PipeRequest, pid: crate::actor::Pid<Vec<u8>>) -> futures::future::BoxFuture<'static, anyhow::Result<()>> {
        use prost::Message as _;

        Box::pin(async move {
            match req.call_id {
                
                    0xd74e => {
                        
                            let input = Zigbee2MqttConfig::decode(req.payload_slice())?;
                            let data = T::config(ctx, input).await?;

                            let out = crate::http::pipe::response_message(req.channel_id, data)?;
                            pid.send(out);

                            Ok(())
                        
                    },
                
                    0x9b83 => {
                        
                            let input = Void::decode(req.payload_slice())?;
                            let data = T::status(ctx, input).await?;

                            let out = crate::http::pipe::response_message(req.channel_id, data)?;
                            pid.send(out);

                            Ok(())
                        
                    },
                
                _ =>  anyhow::bail!("invalid call id {}", req.call_id),
            }
        })
    }
}
