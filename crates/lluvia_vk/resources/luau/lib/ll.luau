--!strict

local compute_node_builder = require("@lib/ll/compute_node_builder.luau")
export type ComptueNodeBuidler = compute_node_builder.ComputeNodeBuilder

export type ContainerNodeBuilder = {
	name: string,
	description: string,

	build_descriptor: (self: ContainerNodeBuilder, args: Arguments) -> ContainerNodeDescriptor,

	on_node_init: (self: ContainerNodeBuilder, node: ContainerNode) -> nil,
	on_node_record: (self: ContainerNodeBuilder, node: ContainerNode, cmdBuffer: CommandBuffer) -> nil,
}

local module = {
	compute_node_builders = {} :: { [string]: ComptueNodeBuidler },
	container_node_builders = {} :: { [string]: ContainerNodeBuilder },
}

module.PortDirection = {
	In = 0,
	Out = 1,
}

module.PortType = {
	Buffer = 0,
	ImageView = 1,
}

module.PortDescriptor = {
	new = function(binding: number, name: string, direction: number, port_type: number)
		return PortDescriptor(binding, name, direction, port_type)
	end,
}

function module.load_program(path: string): Program
	return load_program(path)
end

function module:register_compute_node_builder(builder: ComptueNodeBuidler)
	self.compute_node_builders[builder.name] = builder
end

function module:get_compute_node_builder(name: string): ComptueNodeBuidler
	return self.compute_node_builders[name]
end

function module:register_container_node_builder(builder: ContainerNodeBuilder)
	self.container_node_builders[builder.name] = builder
end

function module:get_container_node_builder(name: string): ContainerNodeBuilder
	return self.container_node_builders[name]
end

-----------------------------------------------------------
-- LOGGING
-----------------------------------------------------------
function module:log_info(msg: string)
	-- TODO
	print(msg)
end

function module:log_warn(msg: string)
	-- TODO
	print(msg)
end

function module:log_error(msg: string)
	-- TODO
	print(msg)
end

-----------------------------------------------------------
-- ComputeNodeDescriptorBuilder
-----------------------------------------------------------

function module.ComputeNodeDescriptorBuilder(): ComputeNodeDescriptorBuilder
	return ComputeNodeDescriptorBuilder()
end

function module.ContainerNodeDescriptorBuilder(): ContainerNodeDescriptorBuilder
	return ContainerNodeDescriptorBuilder()
end

function module.create_compute_node(builder_name: string, args: Arguments?): ComputeNode
	return create_compute_node(builder_name, args)
end

function module.create_container_node(builder_name: string, args: Arguments?): ContainerNode
	return create_container_node(builder_name, args)
end

function module.create_image_view(
	width: number,
	height: number,
	depth: number,
	channel_count: number,
	channel_type: string
): ImageView
	return create_image_view(width, height, depth, channel_count, channel_type)
end

module.ChannelType = {
	Uint8 = "Uint8",
	Int8 = "Int8",
	Uint16 = "Uint16",
	Int16 = "Int16",
	Float16 = "Float16",
	Uint32 = "Uint32",
	Int32 = "Int32",
	Float32 = "Float32",
}

module.ChannelCount = {
	C1 = 1,
	C2 = 2,
	C3 = 3,
	C4 = 4,
}

module.ImageAddressMode = {
	Repeat = 0,
	MirroredRepeat = 1,
	ClampToEdge = 2,
	ClampToBorder = 3,
}

module.ImageFilterMode = {
	Nearest = 0,
	Linear = 1,
}

module.ImageUsageFlagBits = {
	Storage = 1,
	Sampled = 2,
	TransferSrc = 4,
	TransferDst = 8,
}

module.ImageLayout = {
	General = 0,
	TransferDstOptimal = 1,
	ShaderReadOnlyOptimal = 2,
}

module.ImageDescriptor = {
	new = function(
		depth: number,
		height: number,
		width: number,
		channel_count: number,
		channel_type: string
	): ImageDescriptor
		return create_image_descriptor(depth, height, width, channel_count, channel_type)
	end,
}

module.ImageViewDescriptor = {
	new = function(
		address_mode: number,
		filter_mode: number,
		normalized_coords: boolean,
		is_sampled: boolean
	): ImageViewDescriptor
		return create_image_view_descriptor(address_mode, filter_mode, normalized_coords, is_sampled)
	end,
}

function module.fromBase64(encoded: string): VectorUint8
	return from_base64(encoded)
end

local host_memory = {
	createBuffer = function(self, size: number)
		return create_buffer(size, true)
	end,
}

local device_memory = {
	createImage = function(self, desc: ImageDescriptor)
		return create_image(desc)
	end,
}

function module.getHostMemory()
	return host_memory
end

function module.getDeviceMemory()
	return device_memory
end

function module.createCommandBuffer()
	local builder = create_command_buffer_builder()
	return {
		builder = builder,
		begin = function(self)
			-- no-op
		end,
		changeImageLayout = function(self, image, layout)
			-- no-op
		end,
		copyBufferToImage = function(self, src, dst)
			self.builder:copy_buffer_to_image(src, dst)
		end,
		ends = function(self)
			self.cmd_buffer = self.builder:build()
		end,
	}
end

function module.runCommandBuffer(cmdBuffer)
	run_command_buffer(cmdBuffer.cmd_buffer)
end

return module
