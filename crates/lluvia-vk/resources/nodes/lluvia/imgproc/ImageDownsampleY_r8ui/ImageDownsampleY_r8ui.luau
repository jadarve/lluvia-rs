--!strict

local ll = require("@lib/ll.luau")

type ImageDownsampleYArguments = {
	resolution: UVec2,
}

local builder: ll.ComptueNodeBuidler = {
	name = "lluvia/imgproc/ImageDownsampleY_r8ui",
	description = [[
        Downsamples a gray level image along the Y axis.

		Arguments
		---------

		resolution: UVec2, resolution
			Image resolution of the input in_gray image.
    ]],

	build_descriptor = function(self, args: Arguments)
		local parsed_args = args :: ImageDownsampleYArguments
		local out_height = math.floor(parsed_args.resolution.y / 2)

		local desc = ll.ComputeNodeDescriptorBuilder()
			:function_name("main")
			:global_shape(UVec3(parsed_args.resolution.x, out_height, 1))
			:program(ll.load_program("lluvia/imgproc/ImageDownsampleY_r8ui/ImageDownsampleY_r8ui"))
			:add_port(PortDescriptor(0, "in_gray", ll.PortDirection.In, ll.PortType.ImageView))
			:add_port(PortDescriptor(1, "out_gray", ll.PortDirection.Out, ll.PortType.ImageView))
			:build()

		return desc
	end,

	on_node_init = function(self, node) end,
}

ll:register_compute_node_builder(builder)

return {}
