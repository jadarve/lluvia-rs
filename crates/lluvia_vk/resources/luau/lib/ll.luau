--!strict

local compute_node_builder = require("@lib/ll/compute_node_builder.luau")
export type ComptueNodeBuidler = compute_node_builder.ComputeNodeBuilder

export type ContainerNodeBuilder = {
	name: string,
	description: string,

	build_descriptor: (self: ContainerNodeBuilder, args: Arguments) -> ContainerNodeDescriptor,

	on_node_init: (self: ContainerNodeBuilder, node: ContainerNode) -> nil,
	on_node_record: (self: ContainerNodeBuilder, node: ContainerNode, cmdBuffer: CommandBuffer) -> nil,
}

local module = {
	compute_node_builders = {} :: { [string]: ComptueNodeBuidler },
	container_node_builders = {} :: { [string]: ContainerNodeBuilder },
}

module.PortDirection = {
	In = 0,
	Out = 1,
}

module.PortType = {
	Buffer = 0,
	ImageView = 1,
}

module.PortDescriptor = {
	new = function(binding: number, name: string, direction: number, port_type: number)
		return PortDescriptor(binding, name, direction, port_type)
	end,
}

function module.load_program(path: string): Program
	return load_program(path)
end

function module:register_compute_node_builder(builder: ComptueNodeBuidler)
	self.compute_node_builders[builder.name] = builder
end

function module:get_compute_node_builder(name: string): ComptueNodeBuidler
	return self.compute_node_builders[name]
end

function module:register_container_node_builder(builder: ContainerNodeBuilder)
	self.container_node_builders[builder.name] = builder
end

function module:get_container_node_builder(name: string): ContainerNodeBuilder
	return self.container_node_builders[name]
end

-----------------------------------------------------------
-- LOGGING
-----------------------------------------------------------
function module:log_info(msg: string)
	-- TODO
	print(msg)
end

function module:log_warn(msg: string)
	-- TODO
	print(msg)
end

function module:log_error(msg: string)
	-- TODO
	print(msg)
end

-----------------------------------------------------------
-- ComputeNodeDescriptorBuilder
-----------------------------------------------------------

function module.ComputeNodeDescriptorBuilder(): ComputeNodeDescriptorBuilder
	return ComputeNodeDescriptorBuilder()
end

function module.ContainerNodeDescriptorBuilder(): ContainerNodeDescriptorBuilder
	return ContainerNodeDescriptorBuilder()
end

function module.create_compute_node(builder_name: string, args: Arguments?): ComputeNode
	return create_compute_node(builder_name, args)
end

function module.create_container_node(builder_name: string, args: Arguments?): ContainerNode
	return create_container_node(builder_name, args)
end

return module
