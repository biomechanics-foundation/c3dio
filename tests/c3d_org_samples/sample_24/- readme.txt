This is a C3D file data that contains some very odd data.  The overall format appears to be good
although the data stored in the file is not able to be interpreted by any normal C3D application.

09/23/2006  12:56 PM           517,120 MotionMonitorC3D.c3d

This sample C3D file has blank data labels which will cause most C3D applications to fail as there
is no way to distinguish one 3D point from another.  In addition the parameters define a force
plate but there is not analog data associated with it.

Essentially this is a C3D file with bad parameter data that does not match the data.