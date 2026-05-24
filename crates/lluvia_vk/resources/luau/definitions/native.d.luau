
-----------------------------------------------------------
declare class Vec3
    x: number
    y: number
    z: number
end

declare function Vec3(): Vec3

-----------------------------------------------------------
declare class UVec3
    x: number
    y: number
    z: number
end

declare function UVec3(): UVec3

-----------------------------------------------------------

declare class PortDescriptor
    binding: number
    name: string
    direction: number
    port_type: number
end

declare function PortDescriptor(binding: number, name: string, direction: number, port_type: number)

-----------------------------------------------------------
declare class ComputeNodeDescriptor
    function_name: string
    local_shape: UVec3
    grid_shape: UVec3

    -- function configure_grid_shape(self, global_shape: UVec3)
end

declare function ComputeNodeDescriptor(): ComputeNodeDescriptor

-----------------------------------------------------------
declare class Session
    enable_debug: boolean
end


