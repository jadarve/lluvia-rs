--!strict

local ll = require("@lib/ll.luau")

type ColorMapFloatArguments = {
	resolution: UVec2,
}

local builder: ll.ComptueNodeBuidler = {
	name = "lluvia/viz/colormap/ColorMap_float",
	description = [[
        Maps a float image to RGBA using a 1D colormap.

		Arguments
		---------

		resolution: UVec2
			Image resolution.

		Constants
		---------

		min_value: float. Defaults to 0.0.
			Minimum scalar value.

		max_value: float. Defaults to 1.0.
			Maximum scalar value.

		alpha: float. Defaults to 1.0.
			Alpha value of output RGBA in range [0, 1].

		reverse: float. Defaults to 0.0.
			If 1.0, color map is reversed.
    ]],

	build_descriptor = function(self, args: Arguments)
		local parsed_args = args :: ColorMapFloatArguments

		local desc = ll.ComputeNodeDescriptorBuilder()
			:function_name("main")
			:global_shape(UVec3(parsed_args.resolution.x, parsed_args.resolution.y, 1))
			:program(ll.load_program("lluvia/viz/colormap/ColorMap_float/ColorMap_float"))
			:add_port(PortDescriptor(0, "in_image", ll.PortDirection.In, ll.PortType.ImageView))
			:add_port(PortDescriptor(1, "in_colormap", ll.PortDirection.In, ll.PortType.ImageView))
			:add_port(PortDescriptor(2, "out_rgba", ll.PortDirection.Out, ll.PortType.ImageView))
			:add_constant("min_value", 0.0)
			:add_constant("max_value", 1.0)
			:add_constant("alpha", 1.0)
			:add_constant("reverse", 0.0)
			:build()

		return desc
	end,

	on_node_init = function(self, node)
		local min_value = node:get_constant("min_value")
		local max_value = node:get_constant("max_value")
		local alpha = node:get_constant("alpha")
		local reverse = node:get_constant("reverse")

		local pushConstants = PushConstants()
		pushConstants:push_f32(min_value)
		pushConstants:push_f32(max_value)
		pushConstants:push_f32(alpha)
		pushConstants:push_f32(reverse)
		node.push_constants = pushConstants
	end,
}

ll:register_compute_node_builder(builder)

return {}
