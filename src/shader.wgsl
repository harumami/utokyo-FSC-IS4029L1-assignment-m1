@vertex fn vertex_shader(@builtin(vertex_index) index: u32) -> @builtin(position) vec4<f32> {
    switch index {
        case 0: {
            return vec4<f32>(0.0, 0.5, 0.0, 1.0);
        }
        case 1: {
            return vec4<f32>(-0.5, -0.5, 0.0, 1.0);
        }
        default {
            return vec4<f32>(0.5, -0.5, 0.0, 1.0);
        }
    }
}

@fragment fn fragment_shader(@builtin(position) position: vec4<f32>) -> @location(0) vec4<f32> {
    return vec4<f32>(1.0, 1.0, 1.0, 1.0);
}
