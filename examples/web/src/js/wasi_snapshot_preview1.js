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

/**
 * fd_fdstat_get
 *
 * @param {number} fd
 * @param number} bufPtr
 */
export const fd_fdstat_get = function(fd, bufPtr) {}  // eslint-disable-line

/**
 * path_open
 *
 * @param {number} dirfd
 * @param {number} dirflags
 * @param {number} pathPtr
 * @param {number} pathLen
 * @param {number} oflags
 * @param {BigIntPolyfillType | number} fsRightsBase
 * @param {BigIntPolyfillType | number} fsRightsInheriting
 * @param {number} fsFlags
 * @param {number} fd
 */
export const path_open = function (dirfd, dirflags, pathPtr, pathLen, oflags, fsRightsBase, fsRightsInheriting, fsFlags, fd) {}  // eslint-disable-line

/**
 * fd_fdstat_set_flags
 *
 * @param {number} dirfd
 * @param {number} dirflags
 * @param {number} pathPtr
 * @param {number} pathLen
 * @param {number} oflags
 * @param {BigIntPolyfillType | number} fsRightsBase
 * @param {BigIntPolyfillType | number} fsRightsInheriting
 * @param {number} fsFlags
 * @param {number} fd
 */
export const fd_fdstat_set_flags = function(dirfd, dirflags, pathPtr, pathLen, oflags, fsRightsBase, fsRightsInheriting, fsFlags, fd) {}  // eslint-disable-line

/**
 * fd_read
 *
 * @param {number} fd
 * @param {number} iovs
 * @param {number} iovsLen
 * @param {number} nread
 */
export const fd_read = function(fd, iovs, iovsLen, nread) {}  // eslint-disable-line
