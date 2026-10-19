--strict

local ll = require("@lib/ll.luau")

type RGBA2BGRAArguments = {
	resolution: UVec2,
}

local builder: ll.ComptueNodeBuidler = {
	name = "lluvia/color/RGBA2BGRA",
	description = [[
        Converts an RGBA image to BGRA format.

		Arguments
		---------

		resolution: UVec2, resolution
			Image resolution to process
			
    ]],

	build_descriptor = function(self, args: Arguments)
		local parsed_args = args :: RGBA2BGRAArguments

		local desc = ll.ComputeNodeDescriptorBuilder()
			:function_name("main")
			:global_shape(UVec3(parsed_args.resolution.x, parsed_args.resolution.y, 1))
			:program(ll.load_program("lluvia/color/RGBA2BGRA/RGBA2BGRA"))
			:add_port(PortDescriptor(0, "in_rgba", ll.PortDirection.In, ll.PortType.ImageView))
			:add_port(PortDescriptor(1, "out_bgra", ll.PortDirection.Out, ll.PortType.ImageView))
			:build()

		return desc
	end,

	on_node_init = function(self, node)
	end,
}

ll:register_compute_node_builder(builder)

-- return an empty module so that the file can be required by other nodes
-- for instance to import types
return {}
