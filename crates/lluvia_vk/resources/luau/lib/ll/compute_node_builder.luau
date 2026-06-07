--!strict

export type ComputeNodeBuilder = {
	name: string,
	description: string,

	build_descriptor: (self: ComputeNodeBuilder, args: Arguments) -> ComputeNodeDescriptor,

	-- TODO: receive the ComputeNode. Return something?
	on_node_init: (self: ComputeNodeBuilder, node: ComputeNode) -> nil,
}

return {}
