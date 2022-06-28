#[async_trait::async_trait]
pub trait {{ service_name }} {
    {% for method in methods %}
        {% if method.server_streaming %}
            async fn {{ method.name }}(ctx: crate::actor::Task, res: crate::http::pipe::PipeResponse<{{ method.output_type }}>, input: {{ method.input_type }}) -> anyhow::Result<()>;
        {% else %}
            async fn {{ method.name }}(ctx: crate::actor::Task, input: {{ method.input_type }}) -> anyhow::Result<{{ method.output_type }}>;
        {% endif %}
    {% endfor %}
}

#[derive(Default)]
pub struct {{ service_name }}Router<T> {
    _marker: std::marker::PhantomData<T>,
}

impl<T> super::ServiceRouter for {{ service_name }}Router<T> where T: {{ service_name }} + Send + Sync + Default {
    fn id(&self) -> u16 {
        {{ service_hash }}
    }

    fn route(&self, ctx: crate::actor::Task, req: crate::http::pipe::PipeRequest, pid: crate::actor::Pid<Vec<u8>>) -> futures::future::BoxFuture<'static, anyhow::Result<()>> {
        use prost::Message as _;

        Box::pin(async move {
            match req.call_id {
                {% for method in methods %}
                    {{ method.hash }} => {
                        {% if method.server_streaming %}
                            let res = crate::http::pipe::PipeStreamResponse {
                                channel_id: req.channel_id,
                                service_id: req.service_id,
                                pid,
                            };

                            let input = {{ method.input_type }}::decode(req.payload_slice())?;
                            T::{{ method.name }}(ctx, res, input).await?;

                            Ok(())
                        {% else %}
                            let input = {{ method.input_type }}::decode(req.payload_slice())?;
                            let data = T::{{ method.name }}(ctx, input).await?;

                            let out = crate::http::pipe::response_message(req.channel_id, data)?;
                            pid.send(out);

                            Ok(())
                        {% endif %}
                    },
                {% endfor %}
                _ =>  anyhow::bail!("invalid call id {}", req.call_id),
            }
        })
    }
}
