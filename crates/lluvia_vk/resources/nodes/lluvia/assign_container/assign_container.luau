--!strict

local ll = require("@lib/ll.luau")

type AssignContainerArguments = {
	length: number,
	offset: number?,
}

local builder: ll.ContainerNodeBuilder = {
	name = "lluvia/assign_container",
	description = [[
        This is a simple container node that instantiates an assign compute node inside.
        It is intended to be used as a test case for the container node system.

		Arguments
		---------

		length: u32, required
		  Length of the buffer to process
    ]],

	build_descriptor = function(self, args: Arguments)
		local parsed_args = args :: AssignContainerArguments

		local desc = ll.ContainerNodeDescriptorBuilder()
			:builder_name(self.name)
			:add_port(PortDescriptor(0, "out_buffer", ll.PortDirection.Out, ll.PortType.Buffer))
			:add_constant("length", parsed_args.length)
			:build()

		ll:log_info("assign_container.build_descriptor(): finish")

		return desc
	end,

	on_node_init = function(self: ll.ContainerNodeBuilder, node: ContainerNode)
		local args = node:get_constants() :: AssignContainerArguments
		local length = args.length
		local out_buffer = node:get_port("out_buffer")

		-- Instantiate the inner assign node and pass the length argument through
		local assign_node = ll.create_compute_node("lluvia/assign", { length = length })

		-- Bind the output buffer of the container node to the child node
		assign_node:bind("out_buffer", out_buffer)

		-- Pass custom offset constant from container down to child if present
		local offset = args.offset or 0.0
		assign_node:set_constant("offset", offset)

		assign_node:init()

		-- Add the child node explicitly to the container node
		node:add_node("assign_node", assign_node)
	end,

	-- Called when the node is recorded on a CommandBuffer
	on_node_record = function(self: ll.ContainerNodeBuilder, node: ContainerNode, cmdBuffer: CommandBuffer)
		local assign_node = node:get_node("assign_node")

		if assign_node then
			cmdBuffer:run(assign_node)
		else
			ll:log_error("assign_container.on_node_record(): assign_node not found")
		end
	end,
}

ll:register_container_node_builder(builder)

return {}
