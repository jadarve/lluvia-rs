--!strict

local ll = require("@lib/ll.luau")

type ImageDownsampleXArguments = {
	resolution: UVec2,
}

local builder: ll.ComptueNodeBuidler = {
	name = "lluvia/imgproc/ImageDownsampleX_r8ui",
	description = [[
        Downsamples a gray level image along the X axis.

		Arguments
		---------

		resolution: UVec2, resolution
			Image resolution of the input in_gray image.
    ]],

	build_descriptor = function(self, args: Arguments)
		local parsed_args = args :: ImageDownsampleXArguments
		local out_width = math.floor(parsed_args.resolution.x / 2)

		local desc = ll.ComputeNodeDescriptorBuilder()
			:function_name("main")
			:global_shape(UVec3(out_width, parsed_args.resolution.y, 1))
			:program(ll.load_program("lluvia/imgproc/ImageDownsampleX_r8ui/ImageDownsampleX_r8ui"))
			:add_port(PortDescriptor(0, "in_gray", ll.PortDirection.In, ll.PortType.ImageView))
			:add_port(PortDescriptor(1, "out_gray", ll.PortDirection.Out, ll.PortType.ImageView))
			:build()

		return desc
	end,

	on_node_init = function(self, node) end,
}

ll:register_compute_node_builder(builder)

return {}
