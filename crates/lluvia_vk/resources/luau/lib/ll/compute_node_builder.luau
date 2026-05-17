--!strict

export type ComputeNodeBuilder = {
	name: string,
	description: string,

	-- TODO: I could add template argumentts
	new_descriptor: () -> ComputeNodeDescriptor,

	-- TODO: receive the ComputeNode. Return something?
	on_node_init: () -> nil,
}

return nil
