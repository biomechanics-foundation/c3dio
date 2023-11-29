Two sample C3D files from Motion Analysis Corporation motion capture systems.  The first file
uses a set of group and parameter names that deviate from the C3D standard - errors that were
corrected by subsequent versions of the data collection software.  The first file contains
signed analog data and is stored in the SGI integer format.

05/03/2001  02:05 PM           149,504 MACsample.c3d
06/16/2003  12:32 PM           306,688 newwalk.c3d

MACSAMPLE.C3D is a sample C3D file from a very early implementation of the C3D format by Motion
Analysis Corporation system.  Note the odd group and parameter names that deviate from the C3D 
standard.  The file contains analog data in signed format in SGI INT format.  The file conatins
analog data but does not have FORCE_PLATFORM parameters that describe the force plate location.

NEWWALK.C3D is a later C3D file from a Motion Analysis Corporation system.  This file 
includes the new MANUFACTURER group and conforms to the C3D format definition in all respects.
Contains point and analog data - DEC INT format.