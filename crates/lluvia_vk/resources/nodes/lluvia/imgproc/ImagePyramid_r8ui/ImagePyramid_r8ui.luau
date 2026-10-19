--!strict

local ll = require("@lib/ll.luau")

type ImagePyramidArguments = {
	resolution: UVec2,
	levels: number?,
}

local builder: ll.ContainerNodeBuilder = {
	name = "lluvia/imgproc/ImagePyramid_r8ui",
	description = [[
        Creates an image pyramid from an input gray scale image.

		Arguments
		---------

		resolution: UVec2, resolution
			Image resolution of the input in_gray image.

		levels: number, optional. Default to 1.
			The number of levels to create.
    ]],

	build_descriptor = function(self, args: Arguments)
		local parsed_args = args :: ImagePyramidArguments
		local levels = parsed_args.levels or 1

		local desc = ll.ContainerNodeDescriptorBuilder()
			:builder_name(self.name)
			:add_port(PortDescriptor(0, "in_gray", ll.PortDirection.In, ll.PortType.ImageView))

		-- Add the output ports dynamically based on the number of levels
		for i = 0, levels - 1 do
			desc:add_port(
				PortDescriptor(i + 1, "out_gray_" .. tostring(i), ll.PortDirection.Out, ll.PortType.ImageView)
			)
		end
		-- The last port is the alias to the top level output
		desc:add_port(PortDescriptor(levels + 1, "out_gray", ll.PortDirection.Out, ll.PortType.ImageView))

		desc:add_constant("levels", levels)
		desc:add_constant("resolution_x", parsed_args.resolution.x)
		desc:add_constant("resolution_y", parsed_args.resolution.y)

		return desc:build()
	end,

	on_node_init = function(self: ll.ContainerNodeBuilder, node: ContainerNode)
		local args = node:get_constants()
		local levels = args.levels
		local width = args.resolution_x
		local height = args.resolution_y

		local in_gray = node:get_port("in_gray")
		node:bind("out_gray_0", in_gray)

		if levels < 1 then
			error(self.name .. ": levels must be greater or equal 1, got: " .. tostring(levels))
		end

		if levels == 1 then
			node:bind("out_gray", in_gray)
			return
		end

		local current_in = in_gray
		for i = 1, levels - 1 do
			local downX = ll.create_compute_node("lluvia/imgproc/ImageDownsampleX_r8ui", {
				resolution = UVec2(width, height),
			})
			local next_width = math.floor(width / 2)

			local downY = ll.create_compute_node("lluvia/imgproc/ImageDownsampleY_r8ui", {
				resolution = UVec2(next_width, height),
			})
			local next_height = math.floor(height / 2)

			downX:bind("in_gray", current_in)

			-- Allocate intermediate image for downX output
			local int_img = ll.create_image_view(next_width, height, 1, 1, "Uint8")
			downX:bind("out_gray", int_img)

			downY:bind("in_gray", int_img)

			local out_gray_i = node:get_port("out_gray_" .. tostring(i))
			downY:bind("out_gray", out_gray_i)

			downX:init()
			downY:init()

			node:add_node("ImageDownsampleX_r8ui_" .. tostring(i), downX)
			node:add_node("ImageDownsampleY_r8ui_" .. tostring(i), downY)

			if i == levels - 1 then
				node:bind("out_gray", out_gray_i)
			end

			current_in = out_gray_i
			width = next_width
			height = next_height
		end
	end,

	on_node_record = function(self: ll.ContainerNodeBuilder, node: ContainerNode, cmdBuffer: CommandBuffer)
		local args = node:get_constants()
		local levels = args.levels

		for i = 1, levels - 1 do
			local downX = node:get_node("ImageDownsampleX_r8ui_" .. tostring(i))
			local downY = node:get_node("ImageDownsampleY_r8ui_" .. tostring(i))

			if downX and downY then
				cmdBuffer:run(downX)
				cmdBuffer:run(downY)
			else
				ll:log_error("ImagePyramid.on_node_record(): downX or downY not found for level " .. tostring(i))
			end
		end
	end,
}

ll:register_container_node_builder(builder)

return {}
