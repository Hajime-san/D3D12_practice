struct Input {
	float4 position: POSITION;
	float4 svpos: SV_POSITION;
};

float4 BasicPS(Input input) : SV_TARGET
{
	return float4((float2(0,1) + input.position.xy) * 0.5f, 1, 1);
}
