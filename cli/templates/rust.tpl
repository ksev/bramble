#[async_trait::async_trait]
pub trait {{ service_name }} {
    {% for method in methods %}
        {% if method.server_streaming %}
            async fn {{ method.name }}(input: {{ method.input_type }}) -> anyhow::Result<tokio::sync::mpsc::Receiver<anyhow::Result<{{ method.output_type }}>>>;
        {% else %}
            async fn {{ method.name }}(input: {{ method.input_type }}) -> anyhow::Result<{{ method.output_type }}>;
        {% endif %}
    {% endfor %}
}

#[derive(Default)]
pub struct {{ service_name }}Router<T> {
    _marker: std::marker::PhantomData<T>,
}

#[async_trait::async_trait]
impl<T> super::ServiceRouter for {{ service_name }}Router<T> where T: {{ service_name }} + Send + Sync + Default {
    fn id(&self) -> u16 {
        {{ service_hash }}
    }

    async fn route(&self, req: crate::http::pipe::PipeRequest, tx: tokio::sync::mpsc::Sender<Vec<u8>>) -> anyhow::Result<()> {
        use prost::Message as _;

        match req.call_id {
            {% for method in methods %}
                {{ method.hash }} => {
                    {% if method.server_streaming %}
                        let input = {{ method.input_type }}::decode(req.payload_slice())?;
                        let mut rx = T::listen(input).await?;
                        while let Some(data) = rx.recv().await {
                            let out = match data {
                                Ok(data) => crate::http::pipe::part_message(req.channel_id, data)?,
                                Err(e) => crate::http::pipe::error_message(req.channel_id, &format!("{}", e)),
                            };
                            tx.send(out).await?;
                        }
                        Ok(())
                    {% else %}
                        let input = {{ method.input_type }}::decode(req.payload_slice())?;
                        let data = T::{{ method.name }}(input).await?;
                        let out = crate::http::pipe::response_message(req.channel_id, data)?;
                        tx.send(out).await?;

                        Ok(())
                    {% endif %}
                },
            {% endfor %}
            _ =>  anyhow::bail!("invalid call id {}", req.call_id),
        }
    }
}
