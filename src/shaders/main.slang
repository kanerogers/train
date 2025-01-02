struct VertexOutput {
    float4 position: SV_Position;
}

[shader("vertex")]
VertexOutput vertexMain(
    uint vertexID: SV_VertexID
)
{
    static const float2[] positions = {
        float2(0.0, -0.5),
        float2(0.5, 0.5),
        float2(-0.5, 0.5)
    };

    float4 position = float4(positions[vertexID], 0.0, 1.0);

    VertexOutput output = {
        position,
    };

    return output;
}

[shader("fragment")]
float4 fragmentMain(VertexOutput input) : SV_Target
{
    let color = float4(1.0, 0.0, 0.0, 1.0);
    return color;
}
