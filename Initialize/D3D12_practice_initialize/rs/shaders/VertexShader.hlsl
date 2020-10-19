#include "BasicShaderHeader.hlsli"

Output BasicVS(float4 position : POSITION, float2 uv: TEXCOORD) {
	Output output;
	output.svpos = position;
	output.uv = uv;
	return output;
}
