add_commands!(PointsManage, (points), (points_manage));

check_permission!(POINTS_MANAGE_CHECK, "points.manage");

add_commands!(Points, (top,rank), ());