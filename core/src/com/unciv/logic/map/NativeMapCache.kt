package com.unciv.logic.map

import com.unciv.logic.NativeBridge
import com.unciv.logic.map.tile.Tile

/**
 * Manages the Stateful JNI MapCache inside Rust, exposing
 * Delta updates for whenever a Tile's dynamically-tracked stat changes.
 */
class NativeMapCache(val map: TileMap) {
    var nativePtr: Long = 0

    init {
        if (NativeBridge.isNativeAvailable) {
            val tiles = map.values
            nativePtr = NativeBridge.createMapCache(map.mapParameters.mapSize.width, map.mapParameters.mapSize.height, tiles.size)

            // Pass initial state to Rust
            for (tile in map.values) {
                val idx = tile.zeroBasedIndex
                val ownerId = tile.getOwner()?.civInfo?.civName?.hashCode() ?: -1 // Can replace with better dictionary indexing if needed
                val militaryUnitId = if (tile.militaryUnit != null) 1 else 0
                val civilianUnitId = if (tile.civilianUnit != null) 1 else 0
                val militaryUnitOwnerId = tile.militaryUnit?.civInfo?.civName?.hashCode() ?: -1
                val civilianUnitOwnerId = tile.civilianUnit?.civInfo?.civName?.hashCode() ?: -1
                val roadStatus = tile.roadStatus.ordinal

                NativeBridge.updateTile(
                    nativePtr, idx,
                    tile.isWater, tile.isOcean, tile.getBaseTerrain().name == "Mountain",
                    tile.isCityCenter(), roadStatus, ownerId, militaryUnitOwnerId, civilianUnitOwnerId, militaryUnitId, civilianUnitId
                )

                var n0 = -1; var n1 = -1; var n2 = -1; var n3 = -1; var n4 = -1; var n5 = -1
                var nIndex = 0
                for (neighbor in tile.neighbors) {
                    if (nIndex < 6) {
                        try {
                            val nIdx = neighbor.zeroBasedIndex
                            when (nIndex) {
                                0 -> n0 = nIdx
                                1 -> n1 = nIdx
                                2 -> n2 = nIdx
                                3 -> n3 = nIdx
                                4 -> n4 = nIdx
                                5 -> n5 = nIdx
                            }
                        } catch (e: Exception) {
                            // Leave as -1
                        }
                        nIndex++
                    }
                }
                NativeBridge.setTileNeighbors(nativePtr, idx, n0, n1, n2, n3, n4, n5)
            }
        }
    }

    /**
     * Should be fired whenever a unit moves or a tile improvement finishes.
     */
    fun updateTileDynamicState(tile: Tile) {
        if (!NativeBridge.isNativeAvailable || nativePtr == 0L) return

        val idx = tile.zeroBasedIndex

        val ownerId = tile.getOwner()?.civInfo?.civName?.hashCode() ?: -1
        val militaryUnitId = if (tile.militaryUnit != null) 1 else 0
        val civilianUnitId = if (tile.civilianUnit != null) 1 else 0
        val militaryUnitOwnerId = tile.militaryUnit?.civInfo?.civName?.hashCode() ?: -1
        val civilianUnitOwnerId = tile.civilianUnit?.civInfo?.civName?.hashCode() ?: -1
        val roadStatus = tile.roadStatus.ordinal

        NativeBridge.updateTile(
            nativePtr, idx,
            tile.isWater, tile.isOcean, tile.getBaseTerrain().name == "Mountain",
            tile.isCityCenter(), roadStatus, ownerId, militaryUnitOwnerId, civilianUnitOwnerId, militaryUnitId, civilianUnitId
        )
    }

    private val targetBuffer = ThreadLocal.withInitial { IntArray(1024) }

    fun getPotentialAttackTargets(myCivId: Int, reachableIndices: IntArray, attackRange: Int): IntArray? {
        if (!NativeBridge.isNativeAvailable || nativePtr == 0L) return null
        
        // Use a persistent thread-local buffer if we want to avoid allocations, 
        // but for now a correctly-sized one is fine, since target counts rarely exceed ~200
        val outputBuffer = targetBuffer.get()
        val count = NativeBridge.getPotentialAttackTargets(nativePtr, myCivId, reachableIndices, attackRange, outputBuffer)
        
        return outputBuffer.copyOf(count)
    }

    fun destroy() {
        if (nativePtr != 0L) {
            NativeBridge.destroyMapCache(nativePtr)
            nativePtr = 0L
        }
    }

    protected fun finalize() {
        destroy()
    }
}
