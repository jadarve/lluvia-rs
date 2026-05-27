--!strict

local compute_node_builder = require("@lib/ll/compute_node_builder.luau")
export type ComptueNodeBuidler = compute_node_builder.ComputeNodeBuilder

local module = {
	compute_node_builders = {} :: { [string]: ComptueNodeBuidler },
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
-- ComputeNodeDescriptor
-----------------------------------------------------------
function module.ComputeNodeDescriptor(): ComputeNodeDescriptor
	return ComputeNodeDescriptor()
end

return module
