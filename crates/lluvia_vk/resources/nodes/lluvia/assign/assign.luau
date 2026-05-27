--!strict

local ll = require("@lib/ll.luau")

local builder: ll.ComptueNodeBuidler = {
	name = "assign",
	description = [[
        TODO
    ]],

	new_descriptor = function()
		local desc = ComputeNodeDescriptor()

		desc.function_name = "main"
		desc.local_shape = UVec3(32, 1, 1)
		desc.grid_shape = UVec3(1, 1, 1)
		desc.program = ll.load_program("lluvia/assign/assign.spv")
		-- TODO: program
		-- TODO: builder_name

		desc:add_port(PortDescriptor(0, "out_buffer", ll.PortDirection.Out, ll.PortType.Buffer))

		-- TODO: support parameters
		-- desc:setParameter('camera_model', 0)

		ll:log_info("assign.new_descriptor(): finish")

		return desc
	end,

	on_node_init = function(self, node)
		local out_buffer = node:getPort("out_buffer")
		local N = out_buffer // 4

		node:configureGridShape(UVec3(N, 1, 1))
	end,
}

-- TOTHINK: either register the builder here
ll:register_compute_node_builder(builder)

-- or return it as a module
return builder
