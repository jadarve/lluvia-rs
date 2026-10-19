--!strict

local ll = require("@lib/ll.luau")

type RGBA2HSVAArguments = {
	resolution: UVec2,
}

local builder: ll.ComptueNodeBuidler = {
	name = "lluvia/color/RGBA2HSVA",
	description = [[
        Converts an RGBA image to HSVA color space.

		Arguments
		---------

		resolution: UVec2, resolution
			Image resolution to process
			
		Constants
		---------

		min_chroma: float. Defaults to 0.0.
			The minimum chromacity allowed in the conversion. If the chromacity of a given
			pixel is less than min_chroma, then the hue value is set to 0.
    ]],

	build_descriptor = function(self, args: Arguments)
		local parsed_args = args :: RGBA2HSVAArguments

		local desc = ll.ComputeNodeDescriptorBuilder()
			:function_name("main")
			:global_shape(UVec3(parsed_args.resolution.x, parsed_args.resolution.y, 1))
			:program(ll.load_program("lluvia/color/RGBA2HSVA/RGBA2HSVA"))
			:add_port(PortDescriptor(0, "in_rgba", ll.PortDirection.In, ll.PortType.ImageView))
			:add_port(PortDescriptor(1, "out_hsva", ll.PortDirection.Out, ll.PortType.ImageView))
			:add_constant("min_chroma", 0.0)
			:build()

		return desc
	end,

	on_node_init = function(self, node)
		local min_chroma = node:get_constant("min_chroma")
		local pushConstants = PushConstants()
		pushConstants:push_f32(min_chroma)
		node.push_constants = pushConstants
	end,
}

ll:register_compute_node_builder(builder)

-- return an empty module so that the file can be required by other nodes
-- for instance to import types
return {}
