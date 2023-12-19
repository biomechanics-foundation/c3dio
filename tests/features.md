pg 34, p.2: when parameter or group name is interpreted only the first six characters of the group or parameter name are used, all group names and all parameter names within a group are unique (first six letters need to be unique (case insensitive))

pg 35, p.2: unused bytes at the end of the parameter section are normally filled with 0x00h.

pg 35, p.3: the endian format being in the parameter section is annoying, requires checking parameter section before checking events in header, doesn't allow for header-only parsing, put endian in header somewhere?

pg 36, p.2: when writing files, add software manufacturer and version as c3dio or chiron (discuss), read only parameter section if possible for values

pg 42, p.2: locked parameters, how to include them in c3dio? just add as parameter in Parameter definition or make everything unlocked or remove redundancy so no need for explicit locking, what about writing new files with locking, currently not implemented

pg 42, p.5: don't allow explicit changing of some parameters if changing them would screw things up like SCALE, make list of important non-user related values

pg 43, p.4: implement an ASCII-string type? for names of parameters and groups

pg 43, p.7: parameter data is stored as a 1D set of data with dimensions stored with it: make sure those bytes are ordered correctly, 2D strings are a good test case

pg 44, p.3: parameter name length could be negative to indicate a locked parameter, check that it's parsed correctly

pg 44, p.6: minimum set of parameters are required to have a portable c3d file. ensure this minimum set is included for all saves (should be ok with 'named parameters')

pg 44, p.7: never assume format of any parameter, verify that c3dio is robust to weird data formats in parameters either by erroring correctly and/or trying its best with incorrect data type

pg 50, p.2: POINT:SCALE is the max coordinate value divided by 32000, rescale points before every save? or only rescale if values were changed in the data? how to check?

pg 50, p.3: if the fourth word is negative, default to residual of -1

pg 50, p.5: in the integer format, the camera bits are always the high byte of word 4, check impl, check against sample files

pg 50, p.6: if the residual is zero, then the 3D point has been filtered, interpolated, or modified, this is different than negative, where the point is 'invalid'

pg 50, p.8: if signed integer 3D data then analog data values must also be signed integers, check impl

pg 26, p.5: interpolation gap indicated the maximum missing frames allowed to interpolate over. if the user chooses to interpolate over a range larger than this, the interpolation gap should be increased

pg 51, p.7: floating point scale should also be (maximum coordinate value/32000)

pg 52, p.5: if lines don't intersect when spotting a camera, use the mid-point of the line forming the shortest distance between them, for a two line version

pg 53, p.2: use a least-squares technique to calculate the location of a point in space to minimize distance to the ray

pg 54, p.4: automatically detect when a camera is poorly calibrated or poorly observed based on the camera mask, use only first 7 labeled cameras, if want to use other cameras, relabel new ones to be in range 1-7 to observe them

pg 57, p.1: microphone or other analog devices can be added to the data capture environment
