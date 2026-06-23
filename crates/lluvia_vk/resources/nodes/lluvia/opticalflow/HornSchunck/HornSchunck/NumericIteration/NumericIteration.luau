--!strict

local ll = require("@lib/ll.luau")

type NumericIterationArguments = {
	resolution: UVec2,
}

local builder: ll.ComptueNodeBuidler = {
	name = "lluvia/opticalflow/HornSchunck/HornSchunck/NumericIteration",
	description = [[
        Computes one numeric iteration for computing the optical flow.

        Arguments
        ---------

        resolution: UVec2
            Image resolution to process.
    ]],

	build_descriptor = function(self, args: Arguments)
		local parsed_args = args :: NumericIterationArguments

		local desc = ll.ComputeNodeDescriptorBuilder()
			:function_name("main")
			:global_shape(UVec3(parsed_args.resolution.x, parsed_args.resolution.y, 1))
			:program(ll.load_program("lluvia/opticalflow/HornSchunck/HornSchunck/NumericIteration/NumericIteration"))
			:add_port(PortDescriptor(0, "in_image_params", ll.PortDirection.In, ll.PortType.ImageView))
			:add_port(PortDescriptor(1, "in_flow", ll.PortDirection.In, ll.PortType.ImageView))
			:add_port(PortDescriptor(2, "out_flow", ll.PortDirection.Out, ll.PortType.ImageView))
			:build()

		return desc
	end,

	on_node_init = function(self, node) end,
}

ll:register_compute_node_builder(builder)

return {}
