use std::time::Duration;

use bevy::log;
use bevy::prelude::*;

use crate::info::{AnimationDirection, AsepriteInfo};
use crate::plugin::{Aseprite, AsepriteAsset};

#[derive(Debug, Default, Clone)]
pub struct AsepriteAnimation {
    tag: Option<String>,
    current_frame: usize,
    current_timer: Timer,
    forward: bool,
}

impl AsepriteAnimation {
    pub fn new<T, U>(info: &AsepriteInfo, tag: T) -> Self
    where
        T: Into<Option<U>>,
        U: Into<AsepriteTag>,
    {
        let tag = tag.into().map(|t| t.into().0);

        let (current_frame, current_timer, forward) = tag
            .as_ref()
            .and_then(|tag| {
                info.tags.get(tag).map(|tag| {
                    let (current_frame, forward) = match tag.direction {
                        AnimationDirection::Forward
                        | AnimationDirection::PingPong
                        | AnimationDirection::Unknown(_) => (*tag.range.start() as usize, true),
                        AnimationDirection::Reverse | AnimationDirection::PingPongReverse => {
                            (*tag.range.end() as usize, false)
                        }
                    };
                    let current_timer = Timer::from_seconds(
                        info.frame_durations[current_frame] as f32 / 1000.0,
                        TimerMode::Once,
                    );
                    (current_frame, current_timer, forward)
                })
            })
            .unwrap_or_else(|| {
                let current_frame = 0;
                let forward = true;
                let current_timer = Timer::from_seconds(
                    info.frame_durations[current_frame] as f32 / 1000.0,
                    TimerMode::Once,
                );
                (current_frame, current_timer, forward)
            });

        Self {
            tag,
            current_frame,
            current_timer,
            forward,
        }
    }

    /// Returns whether the frame was changed
    fn update(&mut self, info: &AsepriteInfo, dt: Duration) -> bool {
        if self.is_paused() {
            return false;
        }

        self.current_timer.tick(dt);
        if self.current_timer.is_finished() {
            self.next_frame(info);
            self.current_timer = Timer::from_seconds(
                self.current_frame_duration(info).as_secs_f32(),
                TimerMode::Once,
            );
            true
        } else {
            false
        }
    }

    fn next_frame(&mut self, info: &AsepriteInfo) {
        match &self.tag {
            Some(tag) => {
                let tag = match info.tags.get(tag) {
                    Some(tag) => tag,
                    None => {
                        log::error!("Tag {} wasn't found.", tag);
                        return;
                    }
                };

                match tag.direction {
                    AnimationDirection::Forward => {
                        let next_frame = self.current_frame + 1;
                        if tag.range.contains(&(next_frame as u16)) {
                            self.current_frame = next_frame;
                        } else {
                            self.current_frame = *tag.range.start() as usize;
                        }
                    }
                    AnimationDirection::Reverse => {
                        let next_frame = self.current_frame.checked_sub(1);
                        if let Some(next_frame) = next_frame {
                            if tag.range.contains(&(next_frame as u16)) {
                                self.current_frame = next_frame;
                            } else {
                                self.current_frame = *tag.range.end() as usize;
                            }
                        } else {
                            self.current_frame = *tag.range.end() as usize;
                        }
                    }
                    AnimationDirection::PingPong => {
                        if self.forward {
                            let next_frame = self.current_frame + 1;
                            if tag.range.contains(&(next_frame as u16)) {
                                self.current_frame = next_frame;
                            } else {
                                self.current_frame = next_frame.saturating_sub(1);
                                self.forward = false;
                            }
                        } else {
                            let next_frame = self.current_frame.checked_sub(1);
                            if let Some(next_frame) = next_frame
                                && tag.range.contains(&(next_frame as u16))
                            {
                                self.current_frame = next_frame
                            }
                            self.current_frame += 1;
                            self.forward = true;
                        }
                    }
                    AnimationDirection::PingPongReverse => {
                        if self.forward {
                            let next_frame = self.current_frame.checked_sub(1);
                            if let Some(next_frame) = next_frame
                                && tag.range.contains(&(next_frame as u16))
                            {
                                self.current_frame = next_frame
                            }
                            self.current_frame += 1;
                            self.forward = false;
                        } else {
                            let next_frame = self.current_frame + 1;
                            if tag.range.contains(&(next_frame as u16)) {
                                self.current_frame = next_frame;
                            } else {
                                self.current_frame = next_frame.saturating_sub(1);
                                self.forward = true;
                            }
                        }
                    }
                    AnimationDirection::Unknown(dir) => {
                        log::warn!("Unknown animation direction {dir}");
                    }
                }
            }
            None => {
                self.current_frame = (self.current_frame + 1) % info.frame_count;
            }
        }
    }

    /// The current frame duration
    pub fn current_frame_duration(&self, info: &AsepriteInfo) -> Duration {
        Duration::from_millis(info.frame_durations[self.current_frame] as u64)
    }

    /// The current frame absolute index
    pub fn current_frame(&self) -> usize {
        self.current_frame
    }

    /// Set the current frame absolute index
    pub fn set_current_frame(&mut self, frame: usize) {
        self.current_frame = frame
    }

    /// The current frame relative index within the current tag
    pub fn current_tag_frame(&self, info: &AsepriteInfo) -> Option<usize> {
        self.tag.as_ref().and_then(|tag| {
            if let Some(tag) = info.tags.get(tag) {
                Some(
                    self.current_frame
                        .saturating_sub(*tag.range.start() as usize),
                )
            } else {
                log::error!("Tag {} wasn't found.", tag);
                None
            }
        })
    }

    /// Set current frame relative index within the current tag
    pub fn set_current_tag_frame(&mut self, info: &AsepriteInfo, frame: usize) {
        let Some(tag) = self.tag.as_ref().and_then(|tag| info.tags.get(tag)) else {
            return;
        };

        self.current_frame = (*tag.range.start() as usize + frame).min(*tag.range.end() as usize);
    }

    /// The number of reman*ing() range in the current tag
    pub fn remaining_tag_frames(&self, info: &AsepriteInfo) -> Option<usize> {
        self.tag.as_ref().and_then(|tag| {
            if let Some(tag) = info.tags.get(tag) {
                Some((*tag.range.end() as usize) - self.current_frame)
            } else {
                log::error!("Tag {} wasn't found.", tag);
                None
            }
        })
    }

    /// Returns whether the current frame is finished
    pub fn frame_finished(&self, dt: Duration) -> bool {
        self.current_timer.remaining() <= dt
    }

    /// Time elapsed in the current frame
    pub fn time_elapsed(&self) -> Duration {
        self.current_timer.elapsed()
    }

    /// Starts or resumes playing an animation
    pub fn play(&mut self) {
        self.current_timer.unpause()
    }

    /// Pauses the current animation
    pub fn pause(&mut self) {
        self.current_timer.pause()
    }

    /// Returns whether the animation is paused
    pub fn is_paused(&self) -> bool {
        self.current_timer.is_paused()
    }

    /// Returns whether the animation is playing
    pub fn is_playing(&self) -> bool {
        !self.is_paused()
    }

    /// Toggles state between playing and pausing
    pub fn toggle(&mut self) {
        if self.is_paused() {
            self.play()
        } else {
            self.pause()
        }
    }

    pub fn tag(&self) -> Option<&str> {
        self.tag.as_deref()
    }
}

pub fn update_animations(
    time: Res<Time>,
    aseprites: Res<Assets<AsepriteAsset>>,
    mut aseprites_query: Query<(&mut Aseprite, &mut Sprite)>,
) {
    for (mut ase, mut sprite) in aseprites_query.iter_mut() {
        let Some(ase_asset) = aseprites.get(&ase.asset) else {
            log::error!("Aseprite handle {:?}: no corresponding asset", ase.asset);
            continue;
        };
        if ase.anim.update(&ase_asset.info, time.delta()) {
            if let Some(atlas) = sprite.texture_atlas.as_mut() {
                atlas.index = ase.anim.current_frame();
            } else {
                log::error!(
                    "Aseprite handle {:?}: sprite has no texture_atlas",
                    ase.asset
                );
            }
        }
    }
}

pub fn refresh_animations(mut aseprites_query: Query<(&Aseprite, &mut Sprite), Changed<Aseprite>>) {
    for (ase, mut sprite) in aseprites_query.iter_mut() {
        if let Some(atlas) = sprite.texture_atlas.as_mut() {
            atlas.index = ase.anim.current_frame();
        } else {
            log::error!(
                "Aseprite handle {:?}: sprite has no texture_atlas",
                ase.asset
            );
        }
    }
}

/// A tag representing an animation
#[derive(Debug, Default, Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct AsepriteTag(String);

impl From<&str> for AsepriteTag {
    fn from(tag: &str) -> Self {
        Self(tag.to_string())
    }
}

impl From<String> for AsepriteTag {
    fn from(tag: String) -> Self {
        Self(tag)
    }
}

impl std::ops::Deref for AsepriteTag {
    type Target = str;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}
