package com.unciv.logic

import com.badlogic.gdx.Gdx
import com.badlogic.gdx.utils.SharedLibraryLoader
import yairm210.purity.annotations.Readonly

object NativeBridge {
    var isNativeAvailable = false
        private set

    init {
        try {
            SharedLibraryLoader().load("native_bridge")
            isNativeAvailable = true
            try {
                Gdx.app?.debug("NativeBridge", "Successfully loaded native bridge library.")
            } catch (e: Exception) {
                println("Successfully loaded native bridge library.")
            }
        } catch (e: UnsatisfiedLinkError) {
            isNativeAvailable = false
            try {
                Gdx.app?.log("NativeBridge", "Failed to load native bridge library: ${e.message}")
            } catch (e2: Exception) {
                println("Failed to load native bridge library: ${e.message}")
            }
        } catch (e: RuntimeException) {
            isNativeAvailable = false
            try {
                Gdx.app?.log("NativeBridge", "Failed to load native bridge library: ${e.message}")
            } catch (e2: Exception) {
                println("Failed to load native bridge library: ${e.message}")
            }
        }
    }

    @JvmStatic
    external fun hello(): String

    // MapCache endpoints: Handle passing state back to Rust efficiently
    @JvmStatic external fun createMapCache(width: Int, height: Int, tileCount: Int): Long
    @JvmStatic external fun destroyMapCache(ptr: Long)
    @JvmStatic external fun updateTile(ptr: Long, index: Int, isWater: Boolean, isOcean: Boolean, isMountain: Boolean, isCityCenter: Boolean, roadStatus: Int, ownerId: Int, militaryUnitOwnerId: Int, civilianUnitOwnerId: Int, militaryUnitId: Int, civilianUnitId: Int)
    @JvmStatic external fun setTileNeighbors(ptr: Long, index: Int, n0: Int, n1: Int, n2: Int, n3: Int, n4: Int, n5: Int)
    @JvmStatic @Readonly external fun getPotentialAttackTargets(ptr: Long, myCivId: Int, reachableIndices: IntArray, attackRange: Int, outputArray: IntArray): Int

    /**
     * Safe wrapper for the hello endpoint that falls back to Kotlin if native fails.
     */
    fun safeHello(): String {
        return if (isNativeAvailable) {
            try {
                hello()
            } catch (e: Exception) {
                "Hello from Kotlin! (Native threw exception: ${e.message})"
            } catch (e: UnsatisfiedLinkError) {
                "Hello from Kotlin! (Native method missing)"
            }
        } else {
            "Hello from Kotlin! (Native unavailable)"
        }
    }
}
