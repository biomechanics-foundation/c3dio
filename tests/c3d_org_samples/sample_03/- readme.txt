Three sample C3D files containing human gait data generated on an Vicon "370" motion capture system.

04/20/2001  06:12 AM           753,384 gait-pig-nz.c3d
08/07/2000  02:08 PM           242,736 gait-pig.c3d
08/07/2000  02:08 PM           177,744 gait-raw.c3d
  
These are; a normal adult gait trial, the same file after processing to add gait events, kinematics,
and kinetics information.

An additional copy of the output file has been generated with a non-zero start time.

gait-raw.c3d	A DEC, INT file containing a normal adult gait trial. 
gait-pig.c3d	A DEC, INT file containing output from PluginGait containing gait events, kinematics,
		and kinetics information.

gait-pig-nz.c3d A DEC, REAL file with events and a non-zero start time.

Note that the files have FORCE_PLATFORM:ORIGIN information with the force platform origin stored as
a positive value, implying that the mechanical origin of the force plateform is above the surface of
the force plate.