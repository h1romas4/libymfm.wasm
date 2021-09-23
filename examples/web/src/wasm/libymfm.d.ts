/* tslint:disable */
/* eslint-disable */
/**
*/
export class WgmPlay {
  free(): void;
/**
*
* constructor
* @param {number} output_sampling_rate
* @param {number} output_sample_chunk_size
* @param {number} data_length
*/
  constructor(output_sampling_rate: number, output_sample_chunk_size: number, data_length: number);
/**
*
* Return vgmdata buffer referance.
* @returns {number}
*/
  get_seq_data_ref(): number;
/**
*
* Return sampling_l buffer referance.
* @returns {number}
*/
  get_sampling_l_ref(): number;
/**
*
* Return sampling_r buffer referance.
* @returns {number}
*/
  get_sampling_r_ref(): number;
/**
*
* get_header
* @returns {string}
*/
  get_seq_header(): string;
/**
*
* get_gd3
* @returns {string}
*/
  get_seq_gd3(): string;
/**
*
* Initialize sound driver.
*
* # Arguments
* sample_rate - WebAudio sampling rate
* @returns {boolean}
*/
  init(): boolean;
/**
*
* play
* @returns {number}
*/
  play(): number;
}
