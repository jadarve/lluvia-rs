--!strict

local compute_node_builder = require("@lib/ll/compute_node_builder.luau")
export type ComptueNodeBuidler = compute_node_builder.ComputeNodeBuilder

local module = {
	compute_node_builders = {} :: { [string]: ComptueNodeBuidler },
}

function module:register_compute_node_builder(builder: ComptueNodeBuidler)
	self.compute_node_builders[builder.name] = builder
end

function module:get_compute_node_builder(name: string): ComptueNodeBuidler
	return self.compute_node_builders[name]
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
-- ComputeNodeDescriptor
-----------------------------------------------------------
function module.ComputeNodeDescriptor(): ComputeNodeDescriptor
	return ComputeNodeDescriptor()
end

-----------------------------------------------------------
-- Math
-----------------------------------------------------------
-- function module.Vec3()
-- 	return Vec3()
-- end

-- function module.UVec3()
-- 	return UVec3()
-- end

function module.Vec3(x: number, y: number, z: number)
	local v = Vec3()
	v.x = x
	v.y = y
	v.z = z
	return v
end

function module.UVec3(x: number, y: number, z: number)
	local v = UVec3()
	v.x = x
	v.y = y
	v.z = z
	return v
end

return module
