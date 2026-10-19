--!strict

local ll = require("@lib/ll.luau")

type AssignArguments = {
	length: number,
	offset: number?, -- FIXME: nullable properties break Arguments type.
}

local builder: ll.ComptueNodeBuidler = {
	name = "assign",
	description = [[
        This is a simple compute node that assigns the index of the element to the element.
        It is intended to be used as a simple test case for the node system.

		Arguments
		---------

		length: u32, required
		  Length of the buffer to process

		offset: f32, optional. Default to 0.0
		  Offset to the values to assign.
    ]],

	build_descriptor = function(self, args: Arguments)
		-- reinterpret arguments
		local parsed_args = args :: AssignArguments

		local offset = parsed_args.offset or 0.0

		-- TODO: Still need to make sure parse_args actually contains length

		local desc = ll.ComputeNodeDescriptorBuilder()
			:function_name("main")
			:global_shape(UVec3(parsed_args.length, 1, 1))
			:program(ll.load_program("lluvia/assign/assign.spv"))
			:add_port(PortDescriptor(0, "out_buffer", ll.PortDirection.Out, ll.PortType.Buffer))
			:add_constant("offset", offset)
			:build()

		ll:log_info("assign.build_descriptor(): finish")

		return desc
	end,

	on_node_init = function(self, node)
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
