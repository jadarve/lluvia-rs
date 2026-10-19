--!strict

local ll = require("@lib/ll.luau")

type Flow2RGBAArguments = {
	resolution: UVec2,
}

local builder: ll.ComptueNodeBuidler = {
	name = "lluvia/viz/Flow2RGBA",
	description = [[
        Encodes a 2D optical flow field as RGBA color.

		Arguments
		---------

		resolution: UVec2, resolution
			Image resolution to process.

		Constants
		---------

		max_flow: float. Defaults to 1.0.
			The maximum norm of any vector in the field.
    ]],

	build_descriptor = function(self, args: Arguments)
		local parsed_args = args :: Flow2RGBAArguments

		local desc = ll.ComputeNodeDescriptorBuilder()
			:function_name("main")
			:global_shape(UVec3(parsed_args.resolution.x, parsed_args.resolution.y, 1))
			:program(ll.load_program("lluvia/viz/Flow2RGBA/Flow2RGBA"))
			:add_port(PortDescriptor(0, "in_flow", ll.PortDirection.In, ll.PortType.ImageView))
			:add_port(PortDescriptor(1, "out_rgba", ll.PortDirection.Out, ll.PortType.ImageView))
			:add_constant("max_flow", 1.0)
			:build()

		return desc
	end,

	on_node_init = function(self, node)
		local max_flow = node:get_constant("max_flow")
		local pushConstants = PushConstants()
		pushConstants:push_f32(max_flow)
		node.push_constants = pushConstants
	end,
}

ll:register_compute_node_builder(builder)

return {}
