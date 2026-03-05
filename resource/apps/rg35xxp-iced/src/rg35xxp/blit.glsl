struct VertexOutput {
    @builtin(position) position: vec4<f32>,
    @location(0) uv: vec2<f32>,
};

@vertex
fn vs_main(@builtin(vertex_index) in_vertex_index: u32) -> VertexOutput {
    // 生成覆盖全屏的三角形
    var positions = array<vec2<f32>, 3>(
        vec2<f32>(-1.0, -1.0),
        vec2<f32>( 3.0, -1.0),
        vec2<f32>(-1.0,  3.0)
    );

    let pos = positions[in_vertex_index];
    var out: VertexOutput;
    out.position = vec4<f32>(pos, 0.0, 1.0);

    // 修复 UV 计算：
    // 如果之前画面是垂直颠倒的，直接使用线性映射即可，不需要 1.0 - y
    out.uv = vec2<f32>(
        pos.x * 0.5 + 0.5,
        pos.y * 0.5 + 0.5  // <--- 修改了这里，去掉了 "1.0 - (...)"
    );

    return out;
}

@group(0) @binding(0) var t_diffuse: texture_2d<f32>;
@group(0) @binding(1) var s_diffuse: sampler;

@fragment
fn fs_main(in: VertexOutput) -> @location(0) vec4<f32> {
    return textureSample(t_diffuse, s_diffuse, in.uv);
}