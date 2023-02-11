// license:BSD-3-Clause
// copyright-holders:Aaron Giles, Hiromasa Tanaka (for libymfm.wasm)
#include <cmath>
#include <cstdio>
#include <cstdint>
#include <cstring>
#include <list>
#include <string>

#include "ymfm_misc.h"
#include "ymfm_opl.h"
#include "ymfm_opm.h"
#include "ymfm_opn.h"

#define LOG_WRITES (0)

//*********************************************************
//  GLOBAL TYPES
//*********************************************************

// we use an int64_t as emulated time, as a 32.32 fixed point value
using emulated_time = int64_t;

// enumeration of the different types of chips we support
enum chip_type
{
    CHIP_YM2149,
    CHIP_YM2151,
    CHIP_YM2203,
    CHIP_YM2413,
    CHIP_YM2608,
    CHIP_YM2610,
    CHIP_YM2612,
    CHIP_YM3526,
    CHIP_Y8950,
    CHIP_YM3812,
    CHIP_YMF262,
    CHIP_YMF278B,
    CHIP_TYPES
};

//*********************************************************
//  CLASSES
//*********************************************************

// ======================> vgm_chip_base

// abstract base class for a Yamaha chip; we keep a list of these for processing
// as new commands come in
class vgm_chip_base
{
public:
    // construction
    vgm_chip_base(uint32_t clock, chip_type type, char const *name) :
        m_type(type),
        m_name(name)
    {
    }
    virtual ~vgm_chip_base() {}

    // simple getters
    chip_type type() const { return m_type; }
    virtual uint32_t sample_rate() const = 0;

    // required methods for derived classes to implement
    virtual void write(uint32_t reg, uint8_t data) = 0;
    virtual void generate(int32_t *buffer) = 0;

    // write data to the ADPCM-A buffer
    void write_data(ymfm::access_class type, uint32_t base, uint32_t length, uint8_t const *src)
    {
        uint32_t end = base + length;
        if (end > m_data[type].size())
            m_data[type].resize(end);
        memcpy(&m_data[type][base], src, length);
    }

    // seek within the PCM stream
    void seek_pcm(uint32_t pos) { m_pcm_offset = pos; }
    uint8_t read_pcm() { auto &pcm = m_data[ymfm::ACCESS_PCM]; return (m_pcm_offset < pcm.size()) ? pcm[m_pcm_offset++] : 0; }

protected:
    // internal state
    chip_type m_type;
    std::string m_name;
    std::vector<uint8_t> m_data[ymfm::ACCESS_CLASSES];
    uint32_t m_pcm_offset;
};


// ======================> vgm_chip

// actual chip-specific implementation class; includes implementatino of the
// ymfm_interface as needed for vgmplay purposes
template<typename ChipType>
class vgm_chip : public vgm_chip_base, public ymfm::ymfm_interface
{
public:
    // construction
    vgm_chip(uint32_t clock, chip_type type, char const *name) :
        vgm_chip_base(clock, type, name),
        m_chip(*this),
        m_clock(clock),
        m_clocks(0)
    {
        m_chip.reset();
    }

    virtual uint32_t sample_rate() const override
    {
        return m_chip.sample_rate(m_clock);
    }

    // handle a register write: just queue for now
    virtual void write(uint32_t reg, uint8_t data) override
    {
        m_queue.push_back(std::make_pair(reg, data));
    }

    // generate one output sample of output
    virtual void generate(int32_t *buffer) override
    {
        uint32_t addr1 = 0xffff, addr2 = 0xffff;
        uint8_t data1 = 0, data2 = 0;

        // see if there is data to be written; if so, extract it and dequeue
        if (!m_queue.empty())
        {
            auto front = m_queue.front();
            addr1 = 0 + 2 * ((front.first >> 8) & 3);
            data1 = front.first & 0xff;
            addr2 = addr1 + ((m_type == CHIP_YM2149) ? 2 : 1);
            data2 = front.second;
            m_queue.erase(m_queue.begin());
        }

        // write to the chip
        if (addr1 != 0xffff)
        {
            // if (LOG_WRITES)
            //     printf("%10.5f: %s %03X=%02X\n", double(m_clocks) / double(m_chip.sample_rate(m_clock)), m_name.c_str(), data1, data2);
            m_chip.write(addr1, data1);
            m_chip.write(addr2, data2);
        }

        // generate at the appropriate sample rate
        m_chip.generate(&m_output);

        // add the final result to the buffer
        if (m_type == CHIP_YM2203)
        {
            int32_t out0 = m_output.data[0];
            int32_t out1 = m_output.data[1 % ChipType::OUTPUTS];
            int32_t out2 = m_output.data[2 % ChipType::OUTPUTS];
            int32_t out3 = m_output.data[3 % ChipType::OUTPUTS];
            *buffer++ += out0 + (out1 + out2 + out3) / 2;
            *buffer++ += out0 + (out1 + out2 + out3) / 2;
        }
        else if (m_type == CHIP_YM2608 || m_type == CHIP_YM2610)
        {
            int32_t out0 = m_output.data[0];
            int32_t out1 = m_output.data[1 % ChipType::OUTPUTS];
            int32_t out2 = m_output.data[2 % ChipType::OUTPUTS];
            *buffer++ += out0 + (out2 / 2);
            *buffer++ += out1 + (out2 / 2);
        }
        else if (m_type == CHIP_YM2149)
        {
            int32_t out0 = m_output.data[0];
            int32_t out1 = m_output.data[1 % ChipType::OUTPUTS];
            int32_t out2 = m_output.data[2 % ChipType::OUTPUTS];
            *buffer++ += (out0 + out1 + out2) / 2;
            *buffer++ += (out0 + out1 + out2) / 2;
        }
        else if (m_type == CHIP_YMF278B)
        {
            *buffer++ += m_output.data[4];
            *buffer++ += m_output.data[5];
        }
        else if (m_type == CHIP_YM2413)
        {
            int32_t out0 = m_output.data[0];
            int32_t out1 = m_output.data[1 % ChipType::OUTPUTS];
            *buffer++ += out0 + out1;
            *buffer++ += out0 + out1;
        }
        else if (ChipType::OUTPUTS == 1)
        {
            *buffer++ += m_output.data[0] / 2;
            *buffer++ += m_output.data[0] / 2;
        }
        else
        {
            *buffer++ += m_output.data[0];
            *buffer++ += m_output.data[1 % ChipType::OUTPUTS];
        }
        m_clocks++;
    }

protected:
    // handle a read from the buffer
    virtual uint8_t ymfm_external_read(ymfm::access_class type, uint32_t offset) override
    {
        auto &data = m_data[type];
        return (offset < data.size()) ? data[offset] : 0;
    }

    virtual void ymfm_external_write(ymfm::access_class type, uint32_t address, uint8_t data) override
    {
        write_data(type, address, 1, &data);
    }

    // internal state
    ChipType m_chip;
    uint32_t m_clock;
    uint64_t m_clocks;
    typename ChipType::output_data m_output;
    std::vector<std::pair<uint32_t, uint8_t>> m_queue;
};

//*********************************************************
//  GLOBAL HELPERS
//*********************************************************

// global list of active chips
std::list<vgm_chip_base *> active_chips;

template<typename ChipType>
uint32_t add_chips(uint32_t clock, chip_type type, char const *chipname)
{
    uint32_t clockval = clock & 0x3fffffff;
    vgm_chip<ChipType> *chip = new vgm_chip<ChipType>(clockval, type, chipname);
    active_chips.push_back(new vgm_chip<ChipType>(clockval, type, chipname));

    if (type == CHIP_YM2608)
    {
        fprintf(/* for output pcm stdout */ stderr, "load ym2608_adpcm_rom.bin using WASI\n");
        FILE *rom = fopen("ym2608_adpcm_rom.bin", "rb");
        if (rom == nullptr)
            fprintf(stderr, "Warning: YM2608 enabled but ym2608_adpcm_rom.bin not found\n");
        else
        {
            fseek(rom, 0, SEEK_END);
            uint32_t size = ftell(rom);
            fseek(rom, 0, SEEK_SET);
            std::vector<uint8_t> temp(size);
            fread(&temp[0], 1, size, rom);
            fclose(rom);
            for (auto chip : active_chips)
                if (chip->type() == type)
                    chip->write_data(ymfm::ACCESS_ADPCM_A, 0, size, &temp[0]);
        }
    }

    return chip->sample_rate();
}

vgm_chip_base *find_chip(chip_type type, uint8_t index)
{
    for (auto chip : active_chips)
        if (chip->type() == type && index-- == 0)
            return chip;
    return nullptr;
}

void remove_chip(chip_type type, uint8_t index)
{
    vgm_chip_base *chip = find_chip(type, index);
    if(chip != nullptr)
    {
        active_chips.remove(chip);
        delete chip;
    }
}

//*********************************************************
//  FFI interface
//*********************************************************
extern "C" {
uint32_t ymfm_add_chip(uint16_t chip_num, uint32_t clock)
{
    uint32_t sampling_rate = 0;
    switch(chip_num)
    {
        case CHIP_YM2149:
            sampling_rate = add_chips<ymfm::ym2149>(clock, static_cast<chip_type>(chip_num), "YM2149");
            break;
        case CHIP_YM2151:
            sampling_rate = add_chips<ymfm::ym2151>(clock, static_cast<chip_type>(chip_num), "YM2151");
            break;
        case CHIP_YM2203:
            sampling_rate = add_chips<ymfm::ym2203>(clock, static_cast<chip_type>(chip_num), "YM2203");
            break;
        case CHIP_YM2413:
            sampling_rate = add_chips<ymfm::ym2413>(clock, static_cast<chip_type>(chip_num), "YM2413");
            break;
        case CHIP_YM2608:
            sampling_rate = add_chips<ymfm::ym2608>(clock, static_cast<chip_type>(chip_num), "YM2608");
            break;
        case CHIP_YM2610:
            if (clock & 0x80000000)
                sampling_rate = add_chips<ymfm::ym2610b>(clock, static_cast<chip_type>(chip_num), "YM2610B");
            else
                sampling_rate = add_chips<ymfm::ym2610>(clock, static_cast<chip_type>(chip_num), "YM2610");
            break;
        case CHIP_YM2612:
            sampling_rate = add_chips<ymfm::ym2612>(clock, static_cast<chip_type>(chip_num), "YM2612");
            break;
        case CHIP_YM3526:
            sampling_rate = add_chips<ymfm::ym3526>(clock, static_cast<chip_type>(chip_num), "YM3526");
            break;
        case CHIP_Y8950:
            sampling_rate = add_chips<ymfm::y8950>(clock, static_cast<chip_type>(chip_num), "Y8950");
            break;
        case CHIP_YM3812:
            sampling_rate = add_chips<ymfm::ym3812>(clock, static_cast<chip_type>(chip_num), "YM3812");
            break;
        case CHIP_YMF262:
            sampling_rate = add_chips<ymfm::ymf262>(clock, static_cast<chip_type>(chip_num), "YMF262");
            break;
        case CHIP_YMF278B:
            sampling_rate = add_chips<ymfm::ymf278b>(clock, static_cast<chip_type>(chip_num), "YMF278B");
            break;
    }
    return sampling_rate;
}

void ymfm_write(uint16_t chip_num, uint16_t index, uint32_t reg, uint8_t data)
{
    vgm_chip_base* chip = find_chip(static_cast<chip_type>(chip_num), index);
    chip->write(reg, data);
}

void ymfm_generate(uint16_t chip_num, uint16_t index, int32_t *buffer)
{
    vgm_chip_base* chip = find_chip(static_cast<chip_type>(chip_num), index);
    chip->generate(buffer);
}

void ymfm_remove_chip(uint16_t chip_num)
{
    // pop chip
    remove_chip(static_cast<chip_type>(chip_num), 0);
}

// void ymfm_add_rom_data(uint16_t chip_num, uint16_t access_type, uint8_t *buffer, uint32_t length, uint32_t start_address)
void ymfm_add_rom_data(uint16_t chip_num, uint16_t access_type, uint8_t *buffer, uint32_t length, uint32_t start_address)
{
    ymfm::access_class type;
    switch(access_type) {
        case 0x81: // YM2608_DELTA_T
            type = ymfm::ACCESS_ADPCM_B;
            break;
        case 0x82: // YM2610_ADPCM (Also used as YM2608)
            type = ymfm::ACCESS_ADPCM_A;
            break;
        case 0x83: // YM2610_DELTA_T
            type = ymfm::ACCESS_ADPCM_B;
            break;
        case 0x84: // YMF278B_ROM
        case 0x87: // YMF278B_RAM
            type = ymfm::ACCESS_PCM;
            break;
        case 0x88: // Y8950_ROM
            type = ymfm::ACCESS_ADPCM_B;
            break;
    }

    for (int index = 0; index < 2; index++)
    {
        vgm_chip_base *chip = find_chip(static_cast<chip_type>(chip_num), index);
        if(chip != nullptr)
            chip->write_data(type, start_address, length, buffer);
    }
}
} // extern "C"
