package com.example.airplaytv.media

import java.util.ArrayDeque

enum class OfferResult { ACCEPTED, FULL, CLOSED }
class BoundedMediaQueue(private val maxUnits: Int, private val maxBytes: Int) : AutoCloseable {
    private val queue = ArrayDeque<AccessUnit>(); private var bytes = 0; private var closed = false
    init { require(maxUnits > 0 && maxBytes > 0) }
    @Synchronized fun offer(unit: AccessUnit): OfferResult {
        if (closed) { unit.data.close(); return OfferResult.CLOSED }
        if (queue.size >= maxUnits || unit.data.size > maxBytes - bytes) return OfferResult.FULL
        queue.addLast(unit); bytes += unit.data.size; (this as java.lang.Object).notifyAll(); return OfferResult.ACCEPTED
    }
    @Synchronized fun poll(timeoutMs: Long): AccessUnit? {
        require(timeoutMs >= 0); val deadline = System.nanoTime() + timeoutMs * 1_000_000
        while (queue.isEmpty() && !closed) { val left = deadline - System.nanoTime(); if (left <= 0) return null; (this as java.lang.Object).wait((left / 1_000_000).coerceAtLeast(1)) }
        return queue.pollFirst()?.also { bytes -= it.data.size }
    }
    @Synchronized fun flush() { drain() }
    private fun drain() { while (queue.isNotEmpty()) queue.removeFirst().data.close(); bytes = 0 }
    @Synchronized override fun close() { if (!closed) { closed = true; drain(); (this as java.lang.Object).notifyAll() } }
    @Synchronized fun snapshot() = queue.size to bytes
}
