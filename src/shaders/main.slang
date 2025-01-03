struct VertexOutput
{
    float4 position : SV_Position;
}

struct Registers
{
    float4x4 ndc_from_local;
    float4 colour;
}

[vk::push_constant]
uniform Registers registers;

[shader("vertex")]
VertexOutput vertexMain(
    uint vertexID: SV_VertexID)
{
    static const float2[] positions = {
        float2(-0.5, 0.5),  // TOP LEFT
        float2(-0.5, -0.5), // BOTTOM LEFT
        float2(0.5, 0.5),   // TOP RIGHT
        float2(0.5, 0.5),   // TOP RIGHT
        float2(-0.5, -0.5), // BOTTOM LEFT
        float2(0.5, -0.5),  // BOTTOM RIGHT
    };

    float4 position = mul(registers.ndc_from_local, float4(positions[vertexID], 0.0, 1.0));

    VertexOutput output = {
        position,
    };

    return output;
}

[shader("fragment")]
float4 fragmentMain(VertexOutput input)
    : SV_Target
{
    let color = registers.colour;
    return color;
}
