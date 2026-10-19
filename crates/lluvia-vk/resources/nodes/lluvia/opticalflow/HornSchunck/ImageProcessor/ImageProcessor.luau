--!strict

local ll = require("@lib/ll.luau")

type ImageProcessorArguments = {
	resolution: UVec2,
	alpha: number?,
	float_precision: number?,
}

local builder: ll.ComptueNodeBuidler = {
	name = "lluvia/opticalflow/HornSchunck/ImageProcessor",
	description = [[
        Computes the parameters from the input image sequence.

        Arguments
        ---------

        resolution: UVec2
            Image resolution of the input image.

        alpha: float. Defaults to 0.05.
            Regularization gain.

        float_precision: int. Defaults to 0 (FP32).
            Floating point precision used across the algorithm.
    ]],

	build_descriptor = function(self, args: Arguments)
		local parsed_args = args :: ImageProcessorArguments
		local alpha = parsed_args.alpha or 0.05
		local float_precision = parsed_args.float_precision or 0

		local desc = ll.ComputeNodeDescriptorBuilder()
			:function_name("main")
			:global_shape(UVec3(parsed_args.resolution.x, parsed_args.resolution.y, 1))
			:program(ll.load_program("lluvia/opticalflow/HornSchunck/ImageProcessor/ImageProcessor"))
			:add_port(PortDescriptor(0, "in_gray", ll.PortDirection.In, ll.PortType.ImageView))
			:add_port(PortDescriptor(1, "in_gray_old", ll.PortDirection.In, ll.PortType.ImageView))
			:add_port(PortDescriptor(2, "out_image_params", ll.PortDirection.Out, ll.PortType.ImageView))
			:add_constant("alpha", alpha)
			:add_constant("float_precision", float_precision)
			:build()

		return desc
	end,

	on_node_init = function(self, node)
		local alpha = node:get_constant("alpha")
		local pushConstants = PushConstants()
		pushConstants:push_f32(alpha * alpha)
		node.push_constants = pushConstants

		local float_precision = node:get_constant("float_precision")
		local outChannelType = if float_precision == 1 then "Float16" else "Float32"

		local in_gray = node:get_port("in_gray")
		local out_image_params = ll.create_image_view(in_gray.width, in_gray.height, 1, 4, outChannelType)
		node:bind("out_image_params", out_image_params)
	end,
}

ll:register_compute_node_builder(builder)

return {}
