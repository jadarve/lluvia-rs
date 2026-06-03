--!strict

local ll = require("@lib/ll.luau")

local builder: ll.ComptueNodeBuidler = {
	name = "assign",
	description = [[
        This is a simple compute node that assigns the index of the element to the element.
        It is intended to be used as a simple test case for the node system.
    ]],

	new_descriptor = function()
		local desc = ComputeNodeDescriptor()

		desc.function_name = "main"
		desc.local_shape = UVec3(32, 1, 1)
		desc.grid_shape = UVec3(1, 1, 1)
		desc.program = ll.load_program("lluvia/assign/assign.spv")

		desc:add_port(PortDescriptor(0, "out_buffer", ll.PortDirection.Out, ll.PortType.Buffer))

		desc:set_constant("offset", 0.0)

		ll:log_info("assign.new_descriptor(): finish")

		return desc
	end,

	on_node_init = function(self, node)
		local out_buffer = node:get_port("out_buffer")
		local N = out_buffer // 4

		node:configure_grid_shape(UVec3(N, 1, 1))

		local offset = node:get_constant("offset")
		local pushConstants = PushConstants()
		pushConstants:push_f32(offset)
		node.push_constants = pushConstants
	end,
}

ll:register_compute_node_builder(builder)

-- return an empty module so that the file can be required by other nodes
-- for instance to import types
return {}
