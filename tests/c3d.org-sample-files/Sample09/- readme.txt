C3D files containing Vicon gait information

04/01/2002  01:52 PM               517 42798.mp
04/02/2002  03:12 PM           614,864 PlugInC3D.c3d
04/02/2002  03:12 PM           614,864 PolygonC3D.c3d
04/02/2002  03:09 PM           455,344 VER4C3D.c3d

A set of DEC integer C3D files created by Vicon Workstation 4.4 with kinematic processing that has
added additional gait parameters and kinematic variables to the C3D file. Updated to include an
explanation of why Vicon now has to use the REAL (floating point) format for calculated gait parameter
values stored as 3D points.

This collection of four files is a good example of how manufacturers, vendors etc can 
modify C3D files to add extra information.  The first file (VER4C3D.c3d) is the raw data
capture - this is what the Vicon system recorded as the subject walked in the lab.

1.  VER4C3D.c3d     C3D file created by VICON Workstation 4.4
2.  PlugInC3D.c3d   C3D file after processing with PIG (plug in gait).
3.  PolygonC3D.c3d  C3D after using the VICON Polygon report software.
4.  42798.mp        An ASCII text file created by VICON Workstation containing subject parameters.


The second file (PlugInC3D.c3d) contains all the data in the original VER4C3D file
and has been updated to include the results of the Vicon "Plug-In-Gait" calculations.
Unfortunately these are not documented anywhere so they are really only understandable
to the Vicon applications but the structure of the C3D file enables you to figure out
what is going on.  The Video Data block contains the 3-point information and now has
the results of the gait analysis calculations.  If you look at the POINT parameters
then you find the names of the data points (POINT:NAMES) and an optional description
(POINT:DESCRIPTIONS).  The POINT:TYPE_GROUPS parameter, added and only used by Vicon,
gives the names of the POINT: parameters that provide more information about the data.

The first two C3D files illustrate how anyone can add both parameters and data to a C3D file.

Note that there's nothing in the C3D file structure in these files that can
definitively document the status of a "marker" as "real" or "calculated" in an 
undeniable way but you can use the 3D marker residual value and the camera mask 
to make a good guess although I am not confident how reliable this is these days.
The 3D marker residual value is supposed to be zero if the point values has been 
calculated or manufactured in some way, and the camera mask should record which 
of the first six cameras in the lab environment were used to calculate the 
location of any given 3D point.  

So if the camera mask is not zero and the residual value is positive (-1 indicates an 
invalid point) then you can assume that the POINT is a valid 3D marker.  If the camera 
mask is not zero and the residual is zero then the points should be a valid 3D marker 
that has been filtered, interpolated or modified in some way.  Note that both values are 
frame specific - they can be used to determine if a frame is valid or has been modified 
in some way.

A final interesting issue with these files is that they are written in the DEC INTEGER 
format - virtually all C3D files prior to this time are "integer" files that store the 
point co-ordinates as integer values which are then scaled to real-world values via the 
POINT:SCALE parameter.  This works very well for 3D values because the POINT:SCALE value 
is determined by the volume used in the data collection.

However, since the POINT:SCALE value is set by the data collection value it causes a loss 
of resolution when data values are writen as "3D points" but are not 3D points.  As a result 
the calculated gait parameters, e.g. AnkleForce, HipMoment, AnklePower, GroundReactionMoment, 
etc., lose resolution and accuracy when stored as integer values.

Realistically these values should never have been forced into the 3D Point data - the 
manufacturers solution to this problem was in apply the REAL format (floating point) to 
the 3D point data to enable the storage of non-3D point values.  As a result, most VICON
C3D files created since this time are now stored as REAL formatted files and cannot be
converted to integer formatted C3D files without loss of data.

Note that the files have incorrect FORCE_PLATFORM:ORIGIN information frequently seen in 
files created by Oxford Metrics/Vicon systems where the force platform origin is stored as a
positive value, implying that the mechanical origin of the force plateform is above the
surface of the force plate.

