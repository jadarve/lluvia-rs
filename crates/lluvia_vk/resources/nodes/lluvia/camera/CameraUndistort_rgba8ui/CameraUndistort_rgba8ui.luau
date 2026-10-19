--!strict

local ll = require("@lib/ll.luau")

type CameraUndistortArguments = {
	resolution: UVec2,
}

local builder: ll.ComptueNodeBuidler = {
	name = "lluvia/camera/CameraUndistort_rgba8ui",
	description = [[
        Rectifies an RGBA input image applying camera distortion model using Slang.

		Arguments
		---------

		resolution: UVec2, required
			Resolution of the image to rectify.

		Constants
		---------

		camera_model: int. Defaults to 0.
			The camera model used for rectifying the image:
			* 0: standard model
			* 1: fisheye model
    ]],

	build_descriptor = function(self, args: Arguments)
		local parsed_args = args :: CameraUndistortArguments

		local desc = ll.ComputeNodeDescriptorBuilder()
			:function_name("main")
			:global_shape(UVec3(parsed_args.resolution.x, parsed_args.resolution.y, 1))
			:program(ll.load_program("lluvia/camera/CameraUndistort_rgba8ui/CameraUndistort_rgba8ui"))
			:add_port(PortDescriptor(0, "in_rgba", ll.PortDirection.In, ll.PortType.ImageView))
			:add_port(PortDescriptor(1, "in_camera", ll.PortDirection.In, ll.PortType.Buffer))
			:add_port(PortDescriptor(2, "out_rgba", ll.PortDirection.Out, ll.PortType.ImageView))
			:add_constant("camera_model", 0)
			:build()

		return desc
	end,

	on_node_init = function(self, node)
		local camera_model = node:get_constant("camera_model")
		local pushConstants = PushConstants()
		pushConstants:push_i32(camera_model)
		node.push_constants = pushConstants
	end,
}

ll:register_compute_node_builder(builder)

return {}
