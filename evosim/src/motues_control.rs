use std::{cmp::min, f64};

use evotypes_rs::evosoft::motors::MoteusCmd;
//This is meant to be a semi faithful implementation of the control law used by the moteus line
//of motor controllers. 
//The reference code I am using can be found here:
//https://github.com/mjbots/moteus/blob/af2efced994ff4720049ee39def32495535296ce/fw/bldc_servo_position.h#L131
//
struct MotorState {
    control_position: f32,
    control_vel: f32,
    control_accel: f32,
    trajectory_done: bool,

    position: f32,
    velocity: f32,
    torque: f32
}

struct MotorCommand {
    position: f32,
    velocity: f32,

    feedforward_Nm: f32,

    kp: f32,
    kd: f32,
    ilimit: f32,

    velocity_limit: f32,
    accel_limit: f32,

    //there is a lot more here but idk if it is needed for this simulation since I do not plan to
    //support stuff like fixed voltage mode
}

//TODO: implement Default trait for the above.

fn velocity_mode_limit(status: &MotorState, cmd: &mut MotorCommand, rate_hz: f32, velocity: f32) {

    //TODO: Switch these from Option to IsFinite to be more faithful with the original
    //implementation
    if cmd.velocity_limit.is_finite() {
        let dv = cmd.velocity_limit - status.control_vel;
        if cmd.velocity_limit > cmd.velocity {cmd.velocity = cmd.velocity_limit}
        if cmd.velocity_limit < cmd.velocity {cmd.velocity = -cmd.velocity_limit}
    }

    if cmd.accel_limit.is_finite() {
        let dv = velocity - status.control_vel;
        let initial_sign = match dv > 0.0 {
            true => 1.0,
            false => -1.0
        };
        let accel = cmd.accel_limit * initial_sign;

        status.control_accel = accel;
        status.control_vel += accel * todo!(); //Period s
        
        let final_sign = match velocity > status.control_vel {
            true => 1.0,
            false => -1.0
        };
        //TODO: need to figure out what to do for velocity here
        if final_sign != initial_sign {
            status.control_accel = 0.0;
            status.control_vel = velocity;
            status.trajectory_done = true;
        } else {
            status.control_accel = 0.0;
            status.control_vel = velocity;
            status.trajectory_done = true;
        }
    }
}

fn velocity_only_limit(status: &MotorState, cmd: &mut MotorCommand, dx: f32, velocity: f32) {
    let initial_sign = match dx < 0.0 {
        true => 1.0,
        false => -1.0
    };

    status.control_accel = 0.0;
    status.control_vel = -initial_sign * cmd.velocity_limit;
    
    let next_dx = dx - status.control_vel * todo!(); //period s needed
    let final_sign = match next_dx < 0.0 {
        true => 1.0,
        false => -1.0
    };

    if final_sign != initial_sign {
        cmd.position = f32::NAN;
        // data->position_relative_raw.reset(); TODO: Dont know how to translate this yet
        status.control_vel = velocity;
        status.trajectory_done = true;
    }

}

fn calculate_acceleration(
    cmd: &MotorCommand,
    a: f32,
    v0: f32,
    vf: f32,
    dx: f32,
    dv: f32
) -> f32 {
    
    if cmd.velocity_limit.is_finite() {
        return a.copysign(-v0);
    }
    

    let v_frame = v0 - vf;

    if (v_frame * dx) >= 0.0 && dx != 0.0 {
        let decel_distance = (v_frame * v_frame) / (2.0 * a);
        if dx.abs() >= decel_distance {
            if !cmd.velocity_limit.is_finite() || v0.abs() < cmd.velocity_limit  {
                return a.copysign(dx);
            } else {
                return 0.0;
            }
        } else {
            return a.copysign(-v_frame);
        }
    }

    return a.copysign(-v_frame);
}

fn do_velocity_and_accel_limits(rate: f32, cmd: &mut MotorCommand, state: &mut MotorState, velocity: f32) {
    let aO = cmd.accel_limit;
    
    let v0 = state.control_vel;
    let vf = cmd.velocity;

    let dx = cmd.position - state.control_position;
    let dv = vf - v0;
    

    let a = match cmd.accel_limit.is_finite() {
        true => cmd.accel_limit,
        false => return velocity_only_limit(state, cmd, dx, velocity),
    };

    let acceleration = calculate_acceleration(cmd, a, v0, vf, dx, dv);

    let control_accel = acceleration;
    let control_velocity = acceleration * todo!(); //need to figure out what period s is
    let v1 = control_velocity;
    
    let vel_lower = f32::min(v0, v1);
    let vel_upper = f32::max(v0, v1);
    
    if cmd.velocity_limit.is_finite() && vel_upper > cmd.velocity_limit && vel_lower < cmd.velocity_limit {
        control_accel = 0.0;
        state.control_vel = cmd.velocity_limit.copysign(v0);
    } 

    let signed_vel_lower = f32::min(v0, v1);
    let signed_vel_upper = f32::max(v0, v1);

    let target_cross = signed_vel_lower <= vf && signed_vel_upper >= vf;
    let target_near = f32::abs(v1-vf) < (a * 0.5 * period_s);
    let position_near = (f32::abs(dx / vf) <= (10.0 * period_s));

    if ((target_cross || target_near) && position_near) {
        cmd.position = f32::NAN;
        //My command field does not support position reletive raw
        state.control_accel = 0.0;
        state.control_vel = vf;
        state.trajectory_done = true;
    }
}

fn update_trajectory() {
     
}

fn update_command() {
    
}

//Outputs torque
fn position_common() -> f32 {
    //absolute reletive delta
    let velocity_command = update_command();

    //probably going to ignore fixed voltage mode
    //
    let unlimited_torque_Nm = 

    todo!()
}
