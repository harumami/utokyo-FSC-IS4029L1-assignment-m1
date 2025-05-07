struct VertexInput {
    @location(0) position: vec2<f32>,
    @location(1) color: vec3<f32>,
}

struct VertexOutput {
    @builtin(position) position: vec4<f32>,
    @location(0) color: vec3<f32>
}

@vertex fn vertex_main(input: VertexInput) -> VertexOutput {
    var output: VertexOutput;
    output.position = vec4<f32>(input.position, 0.0, 1.0);
    output.color = input.color;
    return output;
}

struct FragmentInput {
    @location(0) color: vec3<f32>,
}

struct FragmentOutput {
    @location(0) color: vec4<f32>,
}

@fragment fn fragment_main(input: FragmentInput) -> FragmentOutput {
    var output: FragmentOutput;
    output.color = vec4<f32>(input.color, 1.0);
    return output;
}
