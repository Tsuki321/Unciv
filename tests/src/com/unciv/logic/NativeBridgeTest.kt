package com.unciv.logic

import com.unciv.logic.map.NativeMapCache
import com.unciv.logic.map.TileMap
import com.unciv.models.metadata.MapParameters
import com.unciv.models.metadata.MapSize
import org.junit.Assert.assertEquals
import org.junit.Assert.assertNotNull
import org.junit.Assert.assertNull
import org.junit.Test
import org.junit.runner.RunWith
import com.unciv.testing.GdxTestRunner

@RunWith(GdxTestRunner::class)
class NativeBridgeTest {

    @Test
    fun `test native bridge fallback`() {
        // Assert safeHello doesn't throw even if library fails to load
        val helloResult = NativeBridge.safeHello()
        assertNotNull("safeHello should return a string regardless of native state", helloResult)
        
        // This ensures the JVM doesn't crash when encountering missing externals
        // and that our try/catch fallback properly handles the state
    }

    @Test
    fun `test NativeMapCache does not crash without active pointer`() {
        val throwawayMapParameters = MapParameters()
        throwawayMapParameters.mapSize = MapSize(10, 10)
        val dummyTileMap = TileMap()
        dummyTileMap.mapParameters = throwawayMapParameters

        val cache = NativeMapCache(dummyTileMap)
        try {
            // If native is missing (e.g. running in pure Java CI without rust DLL), 
            // pointer must be 0 and array endpoints must return null gracefully
            if (!NativeBridge.isNativeAvailable) {
                assertEquals(0L, cache.nativePtr)
                
                val fakeReachable = IntArray(5) { it }
                val result = cache.getPotentialAttackTargets(0, fakeReachable, 2)
                
                assertNull("Fallback mechanism must return null when native bridge is unavailable", result)
            } else {
                // If it is available, ensure we got a valid pointer back and bounds check doesn't crash
                assert(cache.nativePtr != 0L) { "Native pointer should not be 0 when bridge is available" }
                val fakeReachable = IntArray(0)
                val result = cache.getPotentialAttackTargets(0, fakeReachable, 2)
                assertNotNull("Native call should return a valid array when available", result)
            }
        } finally {
            cache.destroy()
            assertEquals(0L, cache.nativePtr)
        }
    }
}
