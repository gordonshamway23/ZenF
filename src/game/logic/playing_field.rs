extern crate alloc;

use super::*;

use agb::fixnum::{Rect, Vector2D};
use alloc::vec::Vec;
use alloc::vec;
use alloc::collections::BTreeSet;

use agb::rng::RandomNumberGenerator;

pub struct PlayingField {
    pub width: usize,
    pub height: usize,
    pub area: usize,
    pub towers: Vec<Tower>,
    pub field: [i32; MAX_PLAYING_FIELD_AREA],
    pub field_solution: [i32; MAX_PLAYING_FIELD_AREA],
    pub mask : Option<PlayingFieldMask>,
}

impl PlayingField {
    pub fn new(width: usize, height: usize, mask : Option<PlayingFieldMask>) -> Self {
        assert!(width > 0 && width <= MAX_PLAYING_FIELD_WIDTH);
        assert!(height > 0 && height <= MAX_PLAYING_FIELD_HEIGHT);
        PlayingField {
            width,
            height,
            area: width * height,
            towers: vec![],
            field: [NO_TOWER; MAX_PLAYING_FIELD_AREA],
            field_solution: [NO_TOWER; MAX_PLAYING_FIELD_AREA],
            mask,
        }
    }

    pub fn clear(&mut self) {
        self.towers.clear();
        for xy in 0..self.area {
            self.field[xy] = NO_TOWER;
            self.field_solution[xy] = NO_TOWER;
        }
    }

    pub fn init_with_random_towers(&mut self, rng : &mut RandomNumberGenerator) {
        self.clear();
        if let Some(m) = &self.mask {
            assert!(m.width==self.width && m.height==self.height);
        }

        let mut empty_tiles_set: BTreeSet<usize> = BTreeSet::new();
        for lin_xy in 0..self.area {
            if let Some(m) = &self.mask {
                if !m.mask[lin_xy] {
                    continue;
                }
            }
            empty_tiles_set.insert(lin_xy);
        }

        let mut new_tower_index = 0;

        
        let mut dir_vecs = vec![(1,0), (-1,0), (0,1), (0,-1)];

        while !empty_tiles_set.is_empty() {
            let new_tower_lin_xy = *empty_tiles_set
                .iter()
                .nth((rng.gen().abs() as usize)%empty_tiles_set.len())
                .unwrap();
            let new_tower_x = new_tower_lin_xy % self.width;
            let new_tower_y = new_tower_lin_xy / self.width;
            let mut new_tower_height : i32 = 1;

            empty_tiles_set.remove(&new_tower_lin_xy);
            self.field_solution[new_tower_lin_xy] = new_tower_index;

            let dir_count: u8 = 1 + (rng.gen().abs()%4) as u8; //1 dir at least, max 4
            let greed: u8 = (rng.gen().abs()%16) as u8;

            crate::util::rng::fisher_yates_shuffle_vec_inplace(&mut dir_vecs, rng);

            for d in 0..dir_count {

                let dv = dir_vecs[d as usize];
                let dvx = dv.0;
                let dvy = dv.1;

                let mut px = new_tower_x as i32;
                let mut py = new_tower_y as i32;

                //see how far we can spread
                let mut ml : i32 = 0; //ml = max length
                loop {
                    px+=dvx;
                    py+=dvy;

                    if px<0 || px>=(self.width as i32) || py<0 || py>=(self.height as i32) {
                        break;
                    }

                    let lin_xy = lin2d!(px as usize, py as usize, self.width);
                    if self.field_solution[lin_xy]!=NO_TOWER {
                        break;
                    }
                    if let Some(m) = &self.mask {
                        if !m.mask[lin_xy] {
                            break;
                        }
                    }

                    ml+=1;
                }

                if ml==0 {
                    continue;
                }

                let l = if (greed & (0b1u8<<d)) != 0u8 {
                    ml
                } else {
                    1+rng.gen().abs()%ml
                };

                //actually spread
                px = new_tower_x as i32;
                py = new_tower_y as i32;
                for _ in 0..l {
                    px+=dvx;
                    py+=dvy;

                    let lin_xy = lin2d!(px as usize, py as usize, self.width);

                    empty_tiles_set.remove(&lin_xy);
                    self.field_solution[lin_xy] = new_tower_index;
                }
                new_tower_height += l;
                
            }

            self.towers.push(Tower {
                x: new_tower_x,
                y: new_tower_y,
                height: new_tower_height,
                flattened_height: new_tower_height,
                bounds: Rect::new(Vector2D::new(new_tower_x as i32, new_tower_y as i32), Vector2D::new(1i32, 1i32)),
            });
            new_tower_index += 1;
        }

        for (tower_index, tower) in self.towers.iter().enumerate()
        {
            self.field[lin2d!(tower.x, tower.y, self.width)]=tower_index as i32;
        }

    }


    pub fn save_as_u8_vec(&self) -> Vec<u8> {
        let mut data = Vec::<u8>::new();

        data.push(self.width as u8);
        data.push(self.height as u8);

        let count_towers_high : u8 = ((self.towers.len()&0xff00)>>8) as u8;
        let count_towers_low : u8 = (self.towers.len()&0xff) as u8;
        data.push(count_towers_high);
        data.push(count_towers_low);

        for tower in self.towers.iter() {
            data.push(tower.x as u8);
            data.push(tower.y as u8);
            data.push(tower.height as u8);
            data.push(tower.flattened_height as u8);
            data.push(tower.bounds.position.x as u8);
            data.push(tower.bounds.position.y as u8);
            data.push(tower.bounds.size.x as u8);
            data.push(tower.bounds.size.y as u8);
        }

        for i in 0..self.width*self.height {
            if self.field[i]==NO_TOWER {
                data.push(255);
                data.push(255);
            } else {
                let field_high = ((self.field[i]&0xff00)>>8) as u8;
                let field_low = (self.field[i]&0xff) as u8;
                data.push(field_high);
                data.push(field_low);
            }
        }
        for i in 0..self.width*self.height {
            if self.field_solution[i]==NO_TOWER {
                data.push(255);
                data.push(255);
            } else {
                let field_solution_high = ((self.field_solution[i]&0xff00)>>8) as u8;
                let field_solution_low = (self.field_solution[i]&0xff) as u8;
                data.push(field_solution_high);
                data.push(field_solution_low);
            }
        }

        data.push(if self.mask.is_some() {1} else {0});
        if let Some(ref mask) = self.mask {
            data.push(mask.width as u8);
            data.push(mask.height as u8);
            for i in 0..mask.width*mask.height {
                data.push(if mask.mask[i] {1} else {0});
            }
        }

        data
    }

    pub fn load_from_u8_vec(&mut self, data : &Vec<u8>) -> usize {
        let mut di : usize=0;
        self.width = data[di] as usize; di+=1;
        self.height = data[di] as usize; di+=1;
        self.area = self.width * self.height;

        let count_towers_high = data[di]; di+=1;
        let count_towers_low = data[di]; di+=1;
        let count_towers = (count_towers_high as usize)<<8 | (count_towers_low as usize);

        self.towers = Vec::<Tower>::with_capacity(count_towers);
        for _ in 0..count_towers {
            self.towers.push(Tower {
                x : data[di+0] as usize,
                y : data[di+1] as usize,
                height : data[di+2] as i32,
                flattened_height : data[di+3] as i32,
                bounds : Rect::<i32> { 
                    position: Vector2D::new(data[di+4] as i32, data[di+5] as i32), 
                    size: Vector2D::new(data[di+6] as i32, data[di+7] as i32) }
            });
            di+=8;  
        }

        self.field = [NO_TOWER; MAX_PLAYING_FIELD_AREA];
        for i in 0..self.width*self.height {
            let field_high = data[di]; di+=1;
            let field_low = data[di]; di+=1;
            if field_high==255 && field_low==255 {
                self.field[i] = NO_TOWER;
            } else {
                self.field[i] = (field_high as i32) << 8 | (field_low as i32);
            }
        }

        self.field_solution = [NO_TOWER; MAX_PLAYING_FIELD_AREA];
        for i in 0..self.width*self.height {
            let field_solution_high = data[di]; di+=1;
            let field_solution_low = data[di]; di+=1;
            if field_solution_high==255 && field_solution_low==255 {
                self.field_solution[i] = NO_TOWER;
            } else {
                self.field_solution[i] = (field_solution_high as i32) << 8 | (field_solution_low as i32);
            }
        }

        self.mask = None;
        let has_mask : bool = data[di]!=0; di+=1;
        if has_mask {
            let mut m = PlayingFieldMask {
                width: data[di+0] as usize,
                height: data[di+1] as usize,
                mask: [true; MAX_PLAYING_FIELD_AREA]
            };
            di+=2;
            for i in 0..m.width*m.height {
                m.mask[i] = data[di]!=0; di+=1;
            }
            self.mask = Some(m)
        }

        di

    }   


    pub fn reset_to_start_state(&mut self) {
        for xy in 0..self.area {
            self.field[xy] = NO_TOWER;
        }
        for (tower_index, tower ) in self.towers.iter_mut().enumerate() {
            tower.flattened_height = tower.height;
            self.field[lin2d!(tower.x, tower.y, self.width)] = tower_index as i32;
        }
    }

    pub fn set_to_solution_state(&mut self) {
        for xy in 0..self.area {
            self.field[xy] = self.field_solution[xy];
        }
        for tower in self.towers.iter_mut() {
            tower.flattened_height = 1
        }
    }

    pub fn is_solved(&self) -> bool {
        for tower in self.towers.iter() {
            if tower.flattened_height>1 {
                return false;
            }
        }
        return true;
    }

    pub fn is_inside(&self, x : i32, y : i32) -> bool {
        x>=0 && x<self.width as i32 && y>=0 && y<self.height as i32
    }
    pub fn is_inside_xy(&self, xy : (i32,i32)) -> bool {
        xy.0>=0 && xy.0<self.width as i32 && xy.1>=0 && xy.1<self.height as i32
    }

    pub fn calc_flatten_xys(&self, tower_xy : (i32,i32), pointing_xy : (i32,i32)) -> Option<Vec<(i32,i32)>> {
        if !self.is_inside_xy(tower_xy) || !self.is_inside_xy(pointing_xy) {
            return None;
        }

        if pointing_xy==tower_xy || (pointing_xy.0!=tower_xy.0 && pointing_xy.1!=tower_xy.1) {
            return None;
        }

        let lin_idx = lin2d!(tower_xy.0 as usize, tower_xy.1 as usize, self.width);

        if let Some(m) = &self.mask {
            if !m.mask[lin_idx] {
                return None;
            }
        }

        let tower_index = self.field[lin_idx];
        if tower_index == NO_TOWER {
            return None;
        }
        
        let tower = &self.towers[tower_index as usize];
        if tower.flattened_height <= 1 {
            return None;
        }

        let dx : i32 = (pointing_xy.0 - tower_xy.0).signum();
        let dy : i32 = (pointing_xy.1 - tower_xy.1).signum();

        let mut x : i32 = tower_xy.0;
        let mut y : i32 = tower_xy.1;

        let mut only_empty_allowed = false;

        let mut flatten_poses : Vec<(i32,i32)> = Vec::with_capacity(self.width.max(self.height));

        let mut height_contingent : i32 = tower.flattened_height - 1;

        loop {
            x += dx;
            y += dy;
            if !self.is_inside(x, y) {
                return None;
            }

            let xy_lin_idx = lin2d!(x as usize, y as usize, self.width);

            if let Some(m) = &self.mask {
                if !m.mask[xy_lin_idx] {
                    return None;
                }
            }

            let xy_tower_index = self.field[xy_lin_idx];

            if xy_tower_index == NO_TOWER {
                only_empty_allowed = true; // only no / empty tower tiles allowed from now on
                flatten_poses.push((x,y));
                height_contingent-=1;
            }
            else if xy_tower_index != tower_index || only_empty_allowed {
                return None;
            }

            if x==pointing_xy.0 && y==pointing_xy.1 {
                return Some(flatten_poses);
            }
            if height_contingent<=0 {
                return None;
            }
        }

    }
    
    pub fn calc_flatten_towards_xys(&self, tower_xy : (i32,i32), dir : (i32,i32), count : i32) -> Option<Vec<(i32,i32)>> {
        assert!(!(dir.0==0 && dir.1==0));
        assert!(count>=1);

        if !self.is_inside_xy(tower_xy) {
            return None;
        }

        let lin_idx = lin2d!(tower_xy.0 as usize, tower_xy.1 as usize, self.width);

        if let Some(m) = &self.mask {
            if !m.mask[lin_idx] {
                return None;
            }
        }

        let tower_index = self.field[lin_idx];
        if tower_index == NO_TOWER {
            return None;
        }
        
        let tower = &self.towers[tower_index as usize];
        if tower.flattened_height <= 1 {
            return None;
        }

        let dx : i32 = dir.0.signum();
        let dy : i32 = dir.1.signum();

        let mut x : i32 = tower_xy.0;
        let mut y : i32 = tower_xy.1;

        let mut only_empty_allowed = false;

        let mut flatten_poses : Vec<(i32,i32)> = Vec::with_capacity(self.width.max(self.height));

        let mut height_contingent : i32 = (tower.flattened_height - 1).min(count);

        loop {
            x += dx;
            y += dy;
            if !self.is_inside(x, y) {
                break;
            }

            let xy_lin_idx = lin2d!(x as usize, y as usize, self.width);

            if let Some(m) = &self.mask {
                if !m.mask[xy_lin_idx] {
                    break;
                }
            }

            let xy_tower_index = self.field[xy_lin_idx];

            if xy_tower_index == NO_TOWER {
                only_empty_allowed = true; // only no / empty tower tiles allowed from now on
                flatten_poses.push((x,y));
                height_contingent-=1;
            }
            else if xy_tower_index != tower_index || only_empty_allowed {
                break;
            }

            if height_contingent<=0 {
                break;
            }
        }

        if flatten_poses.is_empty() {
            None
        } else {
            Some(flatten_poses)
        }

    }

    pub fn flatten(&mut self, tower_xy : (i32,i32), to_flatten_xys : Vec<(i32,i32)>) -> bool {
        if !to_flatten_xys.is_empty() {

            let tower_index = self.field[lin2d!(tower_xy.0 as usize, tower_xy.1 as usize, self.width)];            
            let tower = &mut self.towers[tower_index as usize];

            tower.flattened_height -= to_flatten_xys.len() as i32;

            for fxy in to_flatten_xys.iter() {
                self.field[lin2d!(fxy.0 as usize, fxy.1 as usize, self.width)] = tower_index;
            }

            self.recalc_tower_bounds(tower_xy);

            return true;
        }
        
        return false;
    }


    pub fn calc_deflatten_xys(&self, tower_xy : (i32,i32), pointing_xy : (i32,i32)) -> Option<Vec<(i32,i32)>> {
        if !self.is_inside_xy(tower_xy) || !self.is_inside_xy(pointing_xy) {
            return None;
        }

        if pointing_xy==tower_xy || (pointing_xy.0!=tower_xy.0 && pointing_xy.1!=tower_xy.1) {
            return None;
        }


        let lin_idx = lin2d!(tower_xy.0 as usize, tower_xy.1 as usize, self.width);

        let tower_index = self.field[lin_idx];
        if tower_index == NO_TOWER {
            return None;
        }
        
        let pointing_index = self.field[lin2d!(pointing_xy.0 as usize, pointing_xy.1 as usize, self.width)];
        if pointing_index != tower_index {
            return None;
        }

        //let tower = &self.towers[tower_index as usize];
        //if tower.flattened_height >= tower.height {
        //    return None;
        //}

        let dx : i32 = (pointing_xy.0 - tower_xy.0).signum();
        let dy : i32 = (pointing_xy.1 - tower_xy.1).signum();

        let mut x : i32 = pointing_xy.0;
        let mut y : i32 = pointing_xy.1;

        let mut deflatten_poses : Vec<(i32,i32)> = Vec::with_capacity(self.width.max(self.height));

        deflatten_poses.push((x,y));

        loop {
            x += dx;
            y += dy;
            if !self.is_inside(x, y) {
                break;
            }

            let xy_tower_index = self.field[lin2d!(x as usize, y as usize, self.width)];
            if xy_tower_index!=tower_index {
                break;
            }

            deflatten_poses.push((x,y));

        }
        
        return Some(deflatten_poses);

    }

    pub fn calc_deflatten_towards_xys(&self, tower_xy : (i32,i32), dir : (i32,i32), count : i32) -> Option<Vec<(i32,i32)>> {
        assert!(!(dir.0==0 && dir.1==0));
        assert!(count>=1);

        if !self.is_inside_xy(tower_xy) {
            return None;
        }

        let lin_idx = lin2d!(tower_xy.0 as usize, tower_xy.1 as usize, self.width);

        let tower_index = self.field[lin_idx];
        if tower_index == NO_TOWER {
            return None;
        }
        
        //let tower = &self.towers[tower_index as usize];
        //if tower.flattened_height >= tower.height {
        //    return None;
        //}

        let dx : i32 = dir.0.signum();
        let dy : i32 = dir.1.signum();

        let mut x : i32 = tower_xy.0;
        let mut y : i32 = tower_xy.1;

        let mut deflatten_poses : Vec<(i32,i32)> = Vec::with_capacity(self.width.max(self.height));

        //seek to end
        loop {
            x += dx;
            y += dy;
            if !self.is_inside(x, y) {
                x -= dx;
                y -= dy;
                break;
            }

            let xy_tower_index = self.field[lin2d!(x as usize, y as usize, self.width)];
            if xy_tower_index!=tower_index {
                x -= dx;
                y -= dy;
                break;
            }

        }

        if x==tower_xy.0 && y==tower_xy.1 {
            return None;
        }

        loop {
            deflatten_poses.push((x,y));
            if deflatten_poses.len() >= count as usize {
                break;
            }

            x -= dx;
            y -= dy;

            if x==tower_xy.0 && y==tower_xy.1 {
                break;
            }
        }
        
        return Some(deflatten_poses);

    }

    pub fn deflatten(&mut self, tower_xy : (i32,i32), to_deflatten_xys : Vec<(i32,i32)>) -> bool {
        if !to_deflatten_xys.is_empty() {

            let tower_index = self.field[lin2d!(tower_xy.0 as usize, tower_xy.1 as usize, self.width)];            
            let tower = &mut self.towers[tower_index as usize];

            tower.flattened_height += to_deflatten_xys.len() as i32;

            for dfxy in to_deflatten_xys.iter() {
                self.field[lin2d!(dfxy.0 as usize, dfxy.1 as usize, self.width)] = NO_TOWER;
            }

            self.recalc_tower_bounds(tower_xy);

            return true;
        }
        return false;
    }


    fn recalc_tower_bounds(&mut self, tower_xy : (i32,i32)) {
        if !self.is_inside_xy(tower_xy) {
            return;
        }

        let lin_idx = lin2d!(tower_xy.0 as usize, tower_xy.1 as usize, self.width);

        if let Some(m) = &self.mask {
            if !m.mask[lin_idx] {
                return;
            }
        }

        let tower_index = self.field[lin_idx];
        if tower_index == NO_TOWER {
            return;
        }
        


        let mut x : i32 = tower_xy.0;
        let mut y : i32 = tower_xy.1;

        let mut min_x : i32 = x;
        let mut min_y : i32 = y;
        let mut max_x : i32 = x;
        let mut max_y : i32 = y;

        for (dx,dy) in [(1,0), (-1,0), (0,1), (0,-1)] {
            x = tower_xy.0;
            y = tower_xy.1;

            loop {
                x += dx;
                y += dy;
                if !self.is_inside(x, y) {
                    break;
                }

                let xy_lin_idx = lin2d!(x as usize, y as usize, self.width);

                if let Some(m) = &self.mask {
                    if !m.mask[xy_lin_idx] {
                        break;
                    }
                }

                let xy_tower_index = self.field[xy_lin_idx];

                if xy_tower_index != tower_index {
                    break;
                }

                min_x = min_x.min(x);
                min_y = min_y.min(y);
                max_x = max_x.max(x);
                max_y = max_y.max(y);
            }
        }

        let tower = &mut self.towers[tower_index as usize];
        tower.bounds = Rect::new(Vector2D::new(min_x, min_y), Vector2D::new(max_x-min_x+1, max_y-min_y+1));

    }

}
