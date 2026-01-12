acceleration = trajectory_follower(command_position, command_velocity)

control_velocity = command_velocity OR control_velocity + acceleration * dt OR 0.0
control_position = command_position OR control_position + control_velocity * dt
position_error = control_position - feedback_position
velocity_error = control_velocity - feedback_velocity
position_integrator = limit(position_integrator + ki * position_error * dt, ilimit)

For the sake of simplicity we are going to use the following model:
control_velocity = command_velocity
control_position = command_position OR control_position + control_velocity * dt
position_error = control_position - feedback_position
velocity_error = control_velocity - feedback_velocity
position_integrator = limit(position_integrator + ki * position_error * dt, ilimit)




torque = position_integrator +
         kp * kp_scale * position_error +
         kd * kd_scale * velocity_error +
         command_torque
