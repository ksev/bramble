// ========================== {{ service_name }} ==========================
export class {{ service_name }}Client {
    constructor(private transport: RpcTransport) {}

    {% for method in methods %}
        {% if method.server_streaming %}
            {{ method.name }}(input: {{ method.input_type }}): Readable<{{ method.output_type }}> {
                return derived( 
                    this.transport.stream(
                        {{ service_hash }},
                        {{ method.hash }}, 
                        {{ method.input_type }}.toBinary(input)
                    ), 
                    b => b ? {{ method.output_type }}.fromBinary(b) : null
                );
            }
        {% else %}
            {{ method.name }}(input: {{ method.input_type }}): Promise<{{ method.output_type }}> {
                return this.transport.unary(
                    {{ service_hash }},
                    {{ method.hash }},
                    {{ method.input_type }}.toBinary(input),
                ).then(b => b ? {{ method.output_type }}.fromBinary(b) : null);
            }
        {% endif %}
    {% endfor %}
}
