struct Output {
	float4 position: POSITION;
	float4 svpos: SV_POSITION;
};

Output BasicVS(float4 position : POSITION) {
	Output output;
	output.position = position;
	output.svpos = position;
	return output;
}
