--!strict

local ll = require("@lib/ll.luau")

local builder: ll.ComptueNodeBuidler = {
	name = "lluvia/color/RGBA2Gray",
	description = [[
        Converts an RGBA image to gray scale.
    ]],

	new_descriptor = function()
		local desc = ll.ComputeNodeDescriptorBuilder()
			:function_name("main")
			:global_shape(UVec3(4032, 3024, 1))
			:program(ll.load_program("lluvia/color/RGBA2Gray/RGBA2Gray"))
			:add_port(PortDescriptor(0, "in_rgba", ll.PortDirection.In, ll.PortType.ImageView))
			:add_port(PortDescriptor(1, "out_gray", ll.PortDirection.Out, ll.PortType.ImageView))
			:build()

		return desc
	end,

	on_node_init = function(self, node)

		-- TODO: Need API to get ports in Luau to be able to set grid shape.
		-- ImageView is not bound in Luau yet; grid shape will be configured from Rust
	end,
}

ll:register_compute_node_builder(builder)

-- return an empty module so that the file can be required by other nodes
-- for instance to import types
return {}
