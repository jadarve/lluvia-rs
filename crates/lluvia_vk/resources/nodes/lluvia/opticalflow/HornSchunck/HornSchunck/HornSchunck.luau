--!strict

local ll = require("@lib/ll.luau")

type HornSchunckArguments = {
	resolution: UVec2,
	alpha: number?,
	iterations: number?,
	float_precision: number?,
	clear_history: number?,
}

local builder: ll.ContainerNodeBuilder = {
	name = "lluvia/opticalflow/HornSchunck/HornSchunck",
	description = [[
        Horn and Schunck variational optical flow algorithm.

        Arguments
        ---------

        resolution: UVec2
            Image resolution of the input.

        alpha : float. Defaults to 0.05.
            Regularization gain.

        iterations : int. Defaults to 1.
            Number of iterations run to compute the optical flow.

        float_precision : int. Defaults to 0 (FP32).
            Floating point precision used across the algorithm.

        clear_history : int. Defaults to 1.
            If set to 1, the history of the optical flow is cleared before running the
            algorithm on the next input image.
    ]],

	build_descriptor = function(self, args: Arguments)
		local parsed_args = args :: HornSchunckArguments
		local alpha = parsed_args.alpha or 0.05
		local iterations = parsed_args.iterations or 1
		local float_precision = parsed_args.float_precision or 0
		local clear_history = parsed_args.clear_history or 1

		local desc = ll.ContainerNodeDescriptorBuilder()
			:builder_name(self.name)
			:add_port(PortDescriptor(0, "in_gray", ll.PortDirection.In, ll.PortType.ImageView))
			:add_port(PortDescriptor(1, "out_gray", ll.PortDirection.Out, ll.PortType.ImageView))
			:add_port(PortDescriptor(2, "out_flow", ll.PortDirection.Out, ll.PortType.ImageView))
			:add_constant("resolution_x", parsed_args.resolution.x)
			:add_constant("resolution_y", parsed_args.resolution.y)
			:add_constant("alpha", alpha)
			:add_constant("iterations", iterations)
			:add_constant("float_precision", float_precision)
			:add_constant("clear_history", clear_history)
			:build()

		return desc
	end,

	on_node_init = function(self: ll.ContainerNodeBuilder, node: ContainerNode)
		local args = node:get_constants()
		local width = args.resolution_x
		local height = args.resolution_y
		local alpha = args.alpha
		local iterations = args.iterations
		local float_precision = args.float_precision

		if iterations < 1 then
			error(self.name .. ": iterations must be greater or equal 1, got: " .. tostring(iterations))
		end

		local outChannelType = if float_precision == 1 then "Float16" else "Float32"

		local in_gray = node:get_port("in_gray")

		-- 1. Create the in_gray_old intermediate view
		local in_gray_old = ll.create_image_view(width, height, 1, 1, outChannelType)

		-- 2. Normalization / copy node
		local copyInGrayToInGrayOld = ll.create_compute_node("lluvia/math/normalize/ImageNormalizer_r8ui_r32f", {
			resolution = UVec2(width, height),
		})
		copyInGrayToInGrayOld:bind("in_gray", in_gray)
		copyInGrayToInGrayOld:bind("out_gray", in_gray_old)
		copyInGrayToInGrayOld:set_constant("max_value", 255.0)
		copyInGrayToInGrayOld:init()

		node:add_node("CopyInGrayToInGrayOld", copyInGrayToInGrayOld)

		-- 3. Image processor node
		local imageProcessor = ll.create_compute_node("lluvia/opticalflow/HornSchunck/HornSchunck/ImageProcessor", {
			resolution = UVec2(width, height),
			alpha = alpha,
			float_precision = float_precision,
		})
		imageProcessor:bind("in_gray", in_gray)
		imageProcessor:bind("in_gray_old", in_gray_old)
		imageProcessor:init()

		node:add_node("ImageProcessor", imageProcessor)

		local inImageParams = imageProcessor:get_port("out_image_params")

		-- 4. Allocate flow intermediate views
		local inFlow = ll.create_image_view(width, height, 1, 2, outChannelType)
		local outFlow = ll.create_image_view(width, height, 1, 2, outChannelType)

		-- 5. Numeric iterations
		for i = 1, iterations do
			local numericIteration = ll.create_compute_node("lluvia/opticalflow/HornSchunck/HornSchunck/NumericIteration", {
				resolution = UVec2(width, height),
			})
			numericIteration:bind("in_image_params", inImageParams)
			numericIteration:bind("in_flow", inFlow)
			numericIteration:bind("out_flow", outFlow)
			numericIteration:init()

			node:add_node("NumericIteration_" .. tostring(i), numericIteration)

			inFlow = numericIteration:get_port("out_flow")
			outFlow = numericIteration:get_port("in_flow")
		end

		-- Close the ring of numeric iterations
		local numericIterationFirst = node:get_node("NumericIteration_1")
		local numericIterationLast = node:get_node("NumericIteration_" .. tostring(iterations))
		numericIterationFirst:bind("in_flow", numericIterationLast:get_port("out_flow"))

		-- Bind container outputs
		node:bind("out_flow", numericIterationLast:get_port("out_flow"))
		node:bind("out_gray", in_gray_old)
	end,

	on_node_record = function(self: ll.ContainerNodeBuilder, node: ContainerNode, cmdBuffer: CommandBuffer)
		local args = node:get_constants()
		local iterations = args.iterations

		local imageProcessor = node:get_node("ImageProcessor")
		local copyInGrayToInGrayOld = node:get_node("CopyInGrayToInGrayOld")

		if imageProcessor and copyInGrayToInGrayOld then
			cmdBuffer:run(imageProcessor)
			cmdBuffer:run(copyInGrayToInGrayOld)
		else
			ll:log_error("HornSchunck.on_node_record(): imageProcessor or copyInGrayToInGrayOld not found")
		end

		for i = 1, iterations do
			local numericIteration = node:get_node("NumericIteration_" .. tostring(i))
			if numericIteration then
				cmdBuffer:run(numericIteration)
			else
				ll:log_error("HornSchunck.on_node_record(): NumericIteration_" .. tostring(i) .. " not found")
			end
		end
	end,
}

ll:register_container_node_builder(builder)

return {}
