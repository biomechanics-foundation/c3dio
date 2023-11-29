TYPE-2, TYPE-3, and TYPE-4 force plate data samples.

The TYPE-2 and TYPE-4 two C3D files have been derived from a single source and contain identical force plate data.
The file type-2.c3d describes the force plates as a TYPE-2 plate using only the major diagonal components
of the force plate sensitivity matrix while type-4.c3d describes the force plate as a TYPE-4 plate using
the entire sensitivity matrix.  A copy of the AMTI force plate sensitivity matrix information is included
in a separate file together with an Excel spreadsheet that illustrates the output calculations using the
entire calibration matrix.  These files can be used to verify or test application support for TYPE-4 force
plates.

03/05/2003  05:54 AM             6,202 matrix.pdf
06/02/2002  08:12 PM            72,704 TYPE-2.C3D
03/07/2003  08:57 AM           307,200 TYPE-2a.c3d
02/25/1999  11:06 PM           126,112 TYPE-3.c3d
06/02/2002  08:08 PM            72,704 TYPE-4.C3D
03/07/2003  08:58 AM            50,688 TYPE-4.xls
03/07/2003  07:38 AM           306,688 TYPE-4a.c3d

Updated 7th, March 2003 to correct an error in the calculation and application of the CAL_MATRIX 
which now multiplies the moment columns by 1000.  The previous version (incorrectly) multiplied 
the rows by 1000, following the documentation error in the original AMASS/ADG documentation.

TYPE-2.C3D describes the force plate as a TYPE-2 plate using only the major diagonal components 
of the force plate sensitivity matrix (collected on a Vicon System).

TYPE-2a.C3D describes the force plate as a TYPE-2 plate using only the major diagonal components 
of the force plate sensitivity matrix (collected on a Motion Analysis System).

TYPE-3.C3D contains data as the subject walks across two TYPE-2 and two TYPE-3 force plates.

TYPE-4.C3D describes the force plate as a TYPE-4 plate using the entire sensitivity matrix 
(collected on a Vicon System).

TYPE-4a.C3D describes the force plate as a TYPE-4 plate using the entire sensitivity matrix 
(collected on a Motion Analysis System).

TYPE-4.XLS is a spreadsheet that calculates force and moment outputs from force plate data using 
both TYPE-2 and TYPE-4 calculations and may be used as a referance for calculating TYPE-4 force 
plate outputs.  The spreadsheet also contains a copy of the force plate sensitivity matrix 
information (formerly in MATRIX.XLS) and uses the sensitivity matrix to generate the calibration 
matrix - which in turn creates the CAL_MATRIX.

MATRIX.PDF is a printable copy of the CAL_MATRIX and original sensitivity matrix information used 
in the sample C3D files.

			TYPE-2		TYPE-2a		TYPE-4		TYPE-4a

First Parameter		2		2		2		2
Number of Markers	13		25		13		25
Analog Channels		6		20		6		20
First Frame		1		1		1		1
Last Frame		199		360		199		360
Video Sampling Rate	60.00		60.00		60.00		60.00
Analog Sampling Rate	1200.00		960.00		1200.00		960.00
Scale Factor		0.17		0.06		0.17		0.06
Data Start Record	9		10		9		9
Interpolation Gap	0		0		0		0
C3D File Format		DEC format	DEC format	DEC format	DEC format
Data Format		Integer		Integer		Integer		Integer