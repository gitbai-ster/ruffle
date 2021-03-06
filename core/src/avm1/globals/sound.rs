//! AVM1 Sound object
//! TODO: Sound position, transform, loadSound

use crate::avm1::activation::Activation;
use crate::avm1::error::Error;
use crate::avm1::function::{Executable, FunctionObject};
use crate::avm1::property::Attribute;
use crate::avm1::{Object, SoundObject, TObject, Value};
use crate::avm_warn;
use crate::character::Character;
use crate::display_object::TDisplayObject;
use gc_arena::MutationContext;

/// Implements `Sound`
pub fn constructor<'gc>(
    activation: &mut Activation<'_, 'gc, '_>,
    this: Object<'gc>,
    args: &[Value<'gc>],
) -> Result<Value<'gc>, Error<'gc>> {
    // 1st parameter is the movie clip that "owns" all sounds started by this object.
    // `Sound.setTransform`, `Sound.stop`, etc. will affect all sounds owned by this clip.
    let owner = args
        .get(0)
        .map(|o| o.coerce_to_object(activation))
        .and_then(|o| o.as_display_object());

    if let Some(sound) = this.as_sound_object() {
        sound.set_owner(activation.context.gc_context, owner);
    } else {
        log::error!("Tried to construct a Sound on a non-SoundObject");
    }

    Ok(this.into())
}

pub fn create_proto<'gc>(
    gc_context: MutationContext<'gc, '_>,
    proto: Object<'gc>,
    fn_proto: Object<'gc>,
) -> Object<'gc> {
    let object = SoundObject::empty_sound(gc_context, Some(proto));

    object.as_script_object().unwrap().force_set_function(
        "attachSound",
        attach_sound,
        gc_context,
        Attribute::DONT_DELETE | Attribute::READ_ONLY | Attribute::DONT_ENUM,
        Some(fn_proto),
    );

    object.add_property(
        gc_context,
        "duration",
        FunctionObject::function(
            gc_context,
            Executable::Native(duration),
            Some(fn_proto),
            fn_proto,
        ),
        None,
        Attribute::DONT_DELETE | Attribute::READ_ONLY | Attribute::DONT_ENUM,
    );

    object.add_property(
        gc_context,
        "id3",
        FunctionObject::function(
            gc_context,
            Executable::Native(id3),
            Some(fn_proto),
            fn_proto,
        ),
        None,
        Attribute::DONT_DELETE | Attribute::READ_ONLY | Attribute::DONT_ENUM,
    );

    object.as_script_object().unwrap().force_set_function(
        "getBytesLoaded",
        get_bytes_loaded,
        gc_context,
        Attribute::DONT_DELETE | Attribute::READ_ONLY | Attribute::DONT_ENUM,
        Some(fn_proto),
    );

    object.as_script_object().unwrap().force_set_function(
        "getBytesTotal",
        get_bytes_total,
        gc_context,
        Attribute::DONT_DELETE | Attribute::READ_ONLY | Attribute::DONT_ENUM,
        Some(fn_proto),
    );

    object.as_script_object().unwrap().force_set_function(
        "getPan",
        get_pan,
        gc_context,
        Attribute::DONT_DELETE | Attribute::READ_ONLY | Attribute::DONT_ENUM,
        Some(fn_proto),
    );

    object.as_script_object().unwrap().force_set_function(
        "getTransform",
        get_transform,
        gc_context,
        Attribute::DONT_DELETE | Attribute::READ_ONLY | Attribute::DONT_ENUM,
        Some(fn_proto),
    );

    object.as_script_object().unwrap().force_set_function(
        "getVolume",
        get_volume,
        gc_context,
        Attribute::DONT_DELETE | Attribute::READ_ONLY | Attribute::DONT_ENUM,
        Some(fn_proto),
    );

    object.as_script_object().unwrap().force_set_function(
        "loadSound",
        load_sound,
        gc_context,
        Attribute::DONT_DELETE | Attribute::READ_ONLY | Attribute::DONT_ENUM,
        Some(fn_proto),
    );

    object.add_property(
        gc_context,
        "position",
        FunctionObject::function(
            gc_context,
            Executable::Native(position),
            Some(fn_proto),
            fn_proto,
        ),
        None,
        Attribute::DONT_DELETE | Attribute::READ_ONLY | Attribute::DONT_ENUM,
    );

    object.as_script_object().unwrap().force_set_function(
        "setPan",
        set_pan,
        gc_context,
        Attribute::DONT_DELETE | Attribute::READ_ONLY | Attribute::DONT_ENUM,
        Some(fn_proto),
    );

    object.as_script_object().unwrap().force_set_function(
        "setTransform",
        set_transform,
        gc_context,
        Attribute::DONT_DELETE | Attribute::READ_ONLY | Attribute::DONT_ENUM,
        Some(fn_proto),
    );

    object.as_script_object().unwrap().force_set_function(
        "setVolume",
        set_volume,
        gc_context,
        Attribute::DONT_DELETE | Attribute::READ_ONLY | Attribute::DONT_ENUM,
        Some(fn_proto),
    );

    object.as_script_object().unwrap().force_set_function(
        "start",
        start,
        gc_context,
        Attribute::DONT_DELETE | Attribute::READ_ONLY | Attribute::DONT_ENUM,
        Some(fn_proto),
    );

    object.as_script_object().unwrap().force_set_function(
        "stop",
        stop,
        gc_context,
        Attribute::DONT_DELETE | Attribute::READ_ONLY | Attribute::DONT_ENUM,
        Some(fn_proto),
    );

    object.into()
}

fn attach_sound<'gc>(
    activation: &mut Activation<'_, 'gc, '_>,
    this: Object<'gc>,
    args: &[Value<'gc>],
) -> Result<Value<'gc>, Error<'gc>> {
    let name = args.get(0).unwrap_or(&Value::Undefined);
    if let Some(sound_object) = this.as_sound_object() {
        let name = name.coerce_to_string(activation)?;
        let movie = sound_object
            .owner()
            .or_else(|| activation.context.levels.get(&0).copied())
            .and_then(|o| o.movie());
        if let Some(movie) = movie {
            if let Some(Character::Sound(sound)) = activation
                .context
                .library
                .library_for_movie_mut(movie)
                .character_by_export_name(&name)
            {
                sound_object.set_sound(activation.context.gc_context, Some(*sound));
                sound_object.set_duration(
                    activation.context.gc_context,
                    activation
                        .context
                        .audio
                        .get_sound_duration(*sound)
                        .unwrap_or(0),
                );
                sound_object.set_position(activation.context.gc_context, 0);
            } else {
                avm_warn!(activation, "Sound.attachSound: Sound '{}' not found", name);
            }
        } else {
            avm_warn!(
                activation,
                "Sound.attachSound: Cannot attach Sound '{}' without a library to reference",
                name
            );
        }
    } else {
        avm_warn!(activation, "Sound.attachSound: this is not a Sound");
    }
    Ok(Value::Undefined)
}

fn duration<'gc>(
    activation: &mut Activation<'_, 'gc, '_>,
    this: Object<'gc>,
    _args: &[Value<'gc>],
) -> Result<Value<'gc>, Error<'gc>> {
    if activation.current_swf_version() >= 6 {
        if let Some(sound_object) = this.as_sound_object() {
            return Ok(sound_object.duration().into());
        } else {
            avm_warn!(activation, "Sound.duration: this is not a Sound");
        }
    }

    Ok(Value::Undefined)
}

fn get_bytes_loaded<'gc>(
    activation: &mut Activation<'_, 'gc, '_>,
    _this: Object<'gc>,
    _args: &[Value<'gc>],
) -> Result<Value<'gc>, Error<'gc>> {
    if activation.current_swf_version() >= 6 {
        avm_warn!(activation, "Sound.getBytesLoaded: Unimplemented");
        Ok(1.into())
    } else {
        Ok(Value::Undefined)
    }
}

fn get_bytes_total<'gc>(
    activation: &mut Activation<'_, 'gc, '_>,
    _this: Object<'gc>,
    _args: &[Value<'gc>],
) -> Result<Value<'gc>, Error<'gc>> {
    if activation.current_swf_version() >= 6 {
        avm_warn!(activation, "Sound.getBytesTotal: Unimplemented");
        Ok(1.into())
    } else {
        Ok(Value::Undefined)
    }
}

fn get_pan<'gc>(
    activation: &mut Activation<'_, 'gc, '_>,
    _this: Object<'gc>,
    _args: &[Value<'gc>],
) -> Result<Value<'gc>, Error<'gc>> {
    avm_warn!(activation, "Sound.getPan: Unimplemented");
    Ok(0.into())
}

fn get_transform<'gc>(
    activation: &mut Activation<'_, 'gc, '_>,
    _this: Object<'gc>,
    _args: &[Value<'gc>],
) -> Result<Value<'gc>, Error<'gc>> {
    avm_warn!(activation, "Sound.getTransform: Unimplemented");
    Ok(Value::Undefined)
}

fn get_volume<'gc>(
    activation: &mut Activation<'_, 'gc, '_>,
    _this: Object<'gc>,
    _args: &[Value<'gc>],
) -> Result<Value<'gc>, Error<'gc>> {
    avm_warn!(activation, "Sound.getVolume: Unimplemented");
    Ok(100.into())
}

fn id3<'gc>(
    activation: &mut Activation<'_, 'gc, '_>,
    _this: Object<'gc>,
    _args: &[Value<'gc>],
) -> Result<Value<'gc>, Error<'gc>> {
    if activation.current_swf_version() >= 6 {
        avm_warn!(activation, "Sound.id3: Unimplemented");
    }
    Ok(Value::Undefined)
}

fn load_sound<'gc>(
    activation: &mut Activation<'_, 'gc, '_>,
    _this: Object<'gc>,
    _args: &[Value<'gc>],
) -> Result<Value<'gc>, Error<'gc>> {
    if activation.current_swf_version() >= 6 {
        avm_warn!(activation, "Sound.loadSound: Unimplemented");
    }
    Ok(Value::Undefined)
}

fn position<'gc>(
    activation: &mut Activation<'_, 'gc, '_>,
    this: Object<'gc>,
    _args: &[Value<'gc>],
) -> Result<Value<'gc>, Error<'gc>> {
    if activation.current_swf_version() >= 6 {
        if let Some(sound_object) = this.as_sound_object() {
            // TODO: The position is "sticky"; even if the sound is no longer playing, it should return
            // the previous valid position.
            // Needs some audio backend work for this.
            if sound_object.sound().is_some() {
                if let Some(_sound_instance) = sound_object.sound_instance() {
                    avm_warn!(activation, "Sound.position: Unimplemented");
                }
                return Ok(sound_object.position().into());
            }
        } else {
            avm_warn!(activation, "Sound.position: this is not a Sound");
        }
    }
    Ok(Value::Undefined)
}

fn set_pan<'gc>(
    activation: &mut Activation<'_, 'gc, '_>,
    _this: Object<'gc>,
    _args: &[Value<'gc>],
) -> Result<Value<'gc>, Error<'gc>> {
    avm_warn!(activation, "Sound.setPan: Unimplemented");
    Ok(Value::Undefined)
}

fn set_transform<'gc>(
    activation: &mut Activation<'_, 'gc, '_>,
    _this: Object<'gc>,
    _args: &[Value<'gc>],
) -> Result<Value<'gc>, Error<'gc>> {
    avm_warn!(activation, "Sound.setTransform: Unimplemented");
    Ok(Value::Undefined)
}

fn set_volume<'gc>(
    activation: &mut Activation<'_, 'gc, '_>,
    _this: Object<'gc>,
    _args: &[Value<'gc>],
) -> Result<Value<'gc>, Error<'gc>> {
    avm_warn!(activation, "Sound.setVolume: Unimplemented");
    Ok(Value::Undefined)
}

fn start<'gc>(
    activation: &mut Activation<'_, 'gc, '_>,
    this: Object<'gc>,
    args: &[Value<'gc>],
) -> Result<Value<'gc>, Error<'gc>> {
    let start_offset = args
        .get(0)
        .unwrap_or(&Value::Number(0.0))
        .coerce_to_f64(activation)?;
    let loops = args
        .get(1)
        .unwrap_or(&Value::Number(1.0))
        .coerce_to_f64(activation)?;

    // TODO: Handle loops > std::u16::MAX.
    let loops = (loops as u16).max(1);

    use swf::{SoundEvent, SoundInfo};
    if let Some(sound_object) = this.as_sound_object() {
        if let Some(sound) = sound_object.sound() {
            let sound_instance = activation.context.audio.start_sound(
                sound,
                &SoundInfo {
                    event: SoundEvent::Start,
                    in_sample: if start_offset > 0.0 {
                        Some((start_offset * 44100.0) as u32)
                    } else {
                        None
                    },
                    out_sample: None,
                    num_loops: loops,
                    envelope: None,
                },
            );
            if let Ok(sound_instance) = sound_instance {
                sound_object
                    .set_sound_instance(activation.context.gc_context, Some(sound_instance));
            }
        } else {
            avm_warn!(activation, "Sound.start: No sound is attached");
        }
    } else {
        avm_warn!(activation, "Sound.start: Invalid sound");
    }

    Ok(Value::Undefined)
}

fn stop<'gc>(
    activation: &mut Activation<'_, 'gc, '_>,
    this: Object<'gc>,
    args: &[Value<'gc>],
) -> Result<Value<'gc>, Error<'gc>> {
    if let Some(sound) = this.as_sound_object() {
        if let Some(name) = args.get(0) {
            // Usage 1: Stop all instances of a particular sound, using the name parameter.
            let name = name.coerce_to_string(activation)?;
            let movie = sound
                .owner()
                .or_else(|| activation.context.levels.get(&0).copied())
                .and_then(|o| o.movie());
            if let Some(movie) = movie {
                if let Some(Character::Sound(sound)) = activation
                    .context
                    .library
                    .library_for_movie_mut(movie)
                    .character_by_export_name(&name)
                {
                    // Stop all sounds with the given name.
                    activation.context.audio.stop_sounds_with_handle(*sound);
                } else {
                    avm_warn!(activation, "Sound.stop: Sound '{}' not found", name);
                }
            } else {
                avm_warn!(
                    activation,
                    "Sound.stop: Cannot stop Sound '{}' without a library to reference",
                    name
                )
            }
        } else if let Some(_owner) = sound.owner() {
            // Usage 2: Stop all sound running within a given clip.
            // TODO: We just stop the last played sound for now.
            if let Some(sound_instance) = sound.sound_instance() {
                activation.context.audio.stop_sound(sound_instance);
            }
        } else {
            // Usage 3: If there is no owner and no name, this call acts like `stopAllSounds()`.
            activation.context.audio.stop_all_sounds();
        }
    } else {
        avm_warn!(activation, "Sound.stop: this is not a Sound");
    }

    Ok(Value::Undefined)
}
