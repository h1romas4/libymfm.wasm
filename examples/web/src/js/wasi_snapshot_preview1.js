/**
 * wasi_snapshot_preview1 dummy stab
 */

/**
 * fd_seek
 *
 * @param {number} fd
 * @param {number | bigint} offset
 * @param {number} whence
 * @param {number} newOffsetPtr
 */
export const fd_seek = function(fd, offset, whence, newOffsetPtr) {}; // eslint-disable-line

/**
 * fd_write
 *
 * @param {number} fd
 * @param {number} iovs
 * @param {number} iovsLen
 * @param {number} nwritten
 */
export const fd_write = function(fd, iovs, iovsLen, nwritten) {}; // eslint-disable-line

/**
 * fd_close
 *
 * @param {number} fd
 */
export const fd_close = function(fd) {}; // eslint-disable-line
