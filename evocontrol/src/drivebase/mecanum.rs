struct Mecanum {
    wheel_positions: [evotypes_rs::evosoft::geometry::Pose2d; 4],
    wheel_radius: [f64; 4]
}

impl Mecanum {
    pub fn ik(&self, vector: evotypes_rs::evosoft::geometry::Twist) -> [f64; 4] {
        let fl = (1.0/self.wheel_radius[0]) * (vector.linear.unwrap().x - vector.linear.unwrap().y - (self.wheel_positions[0].x + self.wheel_positions[0].y) * vector.angle.unwrap().z);
        let fr = (1.0/self.wheel_radius[0]) * (vector.linear.unwrap().x + vector.linear.unwrap().y + (self.wheel_positions[1].x + self.wheel_positions[1].y) * vector.angle.unwrap().z);
        let rl = (1.0/self.wheel_radius[0]) * (vector.linear.unwrap().x + vector.linear.unwrap().y - (self.wheel_positions[2].x + self.wheel_positions[2].y) * vector.angle.unwrap().z); 
        let rr = (1.0/self.wheel_radius[0]) * (vector.linear.unwrap().x - vector.linear.unwrap().y + (self.wheel_positions[3].x + self.wheel_positions[3].y) * vector.angle.unwrap().z);

        [fl, fr, rl, rr]
    }

}
