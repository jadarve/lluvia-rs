--!strict

local ll = require("@lib/ll.luau")

type ImageNormalizerArguments = {
	resolution: UVec2,
}

local builder: ll.ComptueNodeBuidler = {
	name = "lluvia/math/normalize/ImageNormalizer_r8ui_r32f",
	description = [[
        Normalizes the content of a 1-channel uint8 (r8ui) gray image and stores it into a 1-channel float32 (r32f) gray image.

        Arguments
        ---------

        resolution: UVec2, resolution
            Image resolution to process.

        Constants
        ---------

        max_value: float. Defaults to 0.0.
            If the value is greater than 0, the input value is divided by max_value and then
            stored in out_gray. Otherwise, the input value is cast to float and stored directly.
    ]],

	build_descriptor = function(self, args: Arguments)
		local parsed_args = args :: ImageNormalizerArguments

		local desc = ll.ComputeNodeDescriptorBuilder()
			:function_name("main")
			:global_shape(UVec3(parsed_args.resolution.x, parsed_args.resolution.y, 1))
			:program(ll.load_program("lluvia/math/normalize/ImageNormalizer_r8ui_r32f/ImageNormalizer_r8ui_r32f"))
			:add_port(PortDescriptor(0, "in_gray", ll.PortDirection.In, ll.PortType.ImageView))
			:add_port(PortDescriptor(1, "out_gray", ll.PortDirection.Out, ll.PortType.ImageView))
			:add_constant("max_value", 0.0)
			:build()

		return desc
	end,

	on_node_init = function(self, node)
		local max_value = node:get_constant("max_value")
		local pushConstants = PushConstants()
		pushConstants:push_f32(max_value)
		node.push_constants = pushConstants
	end,
}

ll:register_compute_node_builder(builder)

return {}
