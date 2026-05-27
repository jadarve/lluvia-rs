--!strict

local ll = require("@lib/ll.luau")

local builder: ll.ComptueNodeBuidler = {
	name = "lluvia/color/RGBA2Gray",
	description = [[
        Converts an RGBA image to gray scale.
    ]],

	new_descriptor = function()
		local desc = ComputeNodeDescriptor()

		desc.function_name = "main"
		desc.local_shape = UVec3(32, 32, 1)
		desc.grid_shape = UVec3(1, 1, 1)
		desc.program = ll.load_program("lluvia/color/RGBA2Gray/RGBA2Gray")

		desc:add_port(PortDescriptor(0, "in_rgba", ll.PortDirection.In, ll.PortType.ImageView))
		desc:add_port(PortDescriptor(1, "out_gray", ll.PortDirection.Out, ll.PortType.ImageView))

		return desc
	end,

	on_node_init = function(self, node)

		-- TODO: Need API to get ports in Luau to be able to set grid shape.
		-- ImageView is not bound in Luau yet; grid shape will be configured from Rust
	end,
}

ll:register_compute_node_builder(builder)

return builder
