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

    // simple getters
    chip_type type() const { return m_type; }
    virtual uint32_t sample_rate() const = 0;

    // required methods for derived classes to implement
    virtual void write(uint32_t reg, uint8_t data) = 0;
    virtual void generate(emulated_time output_start, emulated_time output_step, int32_t *buffer) = 0;

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
        m_clocks(0),
        m_step(0x100000000ull / m_chip.sample_rate(clock)),
        m_pos(0)
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
    virtual void generate(emulated_time output_start, emulated_time output_step, int32_t *buffer) override
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
            if (LOG_WRITES)
                printf("%10.5f: %s %03X=%02X\n", double(m_clocks) / double(m_chip.sample_rate(m_clock)), m_name.c_str(), data1, data2);
            m_chip.write(addr1, data1);
            m_chip.write(addr2, data2);
        }

        // generate at the appropriate sample rate
        for ( ; m_pos <= output_start; m_pos += m_step)
        {
            m_chip.generate(&m_output);
        }

        // add the final result to the buffer
        if (m_type == CHIP_YM2203)
        {
            int32_t out0 = m_output.data[0];
            int32_t out1 = m_output.data[1 % ChipType::OUTPUTS];
            int32_t out2 = m_output.data[2 % ChipType::OUTPUTS];
            int32_t out3 = m_output.data[3 % ChipType::OUTPUTS];
            *buffer++ += out0 + out1 + out2 + out3;
            *buffer++ += out0 + out1 + out2 + out3;
        }
        else if (m_type == CHIP_YM2608 || m_type == CHIP_YM2610)
        {
            int32_t out0 = m_output.data[0];
            int32_t out1 = m_output.data[1 % ChipType::OUTPUTS];
            int32_t out2 = m_output.data[2 % ChipType::OUTPUTS];
            *buffer++ += out0 + out2;
            *buffer++ += out1 + out2;
        }
        else if (m_type == CHIP_YMF278B)
        {
            *buffer++ += m_output.data[4];
            *buffer++ += m_output.data[5];
        }
        else if (ChipType::OUTPUTS == 1)
        {
            *buffer++ += m_output.data[0];
            *buffer++ += m_output.data[0];
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

    // internal state
    ChipType m_chip;
    uint32_t m_clock;
    uint64_t m_clocks;
    typename ChipType::output_data m_output;
    emulated_time m_step;
    emulated_time m_pos;
    std::vector<std::pair<uint32_t, uint8_t>> m_queue;
};

// vgm_chip_base *ym2149;
// vgm_chip_base *ym2203;
// vgm_chip_base *ym2413;
// vgm_chip_base *ym2608;
// vgm_chip_base *ym2610;
// vgm_chip_base *ym2612;
// vgm_chip_base *ym3526;
// vgm_chip_base *ym3812;

vgm_chip_base *ym2151;
vgm_chip_base *ym2203;

extern "C"
{
    void ym2151_init(uint32_t clock) {
        ym2151 = new vgm_chip<ymfm::ym2151>(clock, CHIP_YM2151, "YM2151");
    }

    void ym2151_write(uint32_t reg, uint8_t data) {
        ym2151->write(reg, data);
    }

    void ym2151_generate(int64_t emulated_time, int64_t output_step, int32_t* buffer) {
        ym2151->generate(emulated_time, output_step, buffer);
    }

    void ym2203_init(uint32_t clock) {
        ym2203 = new vgm_chip<ymfm::ym2203>(clock, CHIP_YM2203, "YM2203");
    }

    void ym2203_write(uint32_t reg, uint8_t data) {
        ym2203->write(reg, data);
    }

    void ym2203_generate(int64_t emulated_time, int64_t output_step, int32_t* buffer) {
        ym2203->generate(emulated_time, output_step, buffer);
    }
}
